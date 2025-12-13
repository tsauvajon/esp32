use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Stack};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Instant, Timer};
use esp_radio::wifi::{self, WifiStaState};
use heapless::String;
use log::{info, warn};
use mountain_mqtt::client::{Client, ClientError, ConnectionSettings, EventHandlerError};
use mountain_mqtt::data::quality_of_service::QualityOfService;
use mountain_mqtt::mqtt_manager::{ConnectionId, MqttOperations};
use mountain_mqtt::packets::publish::ApplicationMessage;
use mountain_mqtt_embassy::mqtt_manager::{self, FromApplicationMessage, MqttEvent, Settings};
use sht31::Reading;

const MQTT_BROKER_IP: &str = env!("MQTT_BROKER_IP");
const MQTT_BROKER_PORT: &str = env!("MQTT_BROKER_PORT");
const MQTT_CLIENT_ID: &str = env!("MQTT_CLIENT_ID");
const MQTT_TOPIC_TELEMETRY: &str = env!("MQTT_TOPIC_TELEMETRY");
const MQTT_TOPIC_STATUS: &str = env!("MQTT_TOPIC_STATUS");
const FIRMWARE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

const ACTION_QUEUE: usize = 8;
const EVENT_QUEUE: usize = 8;
const MQTT_BUFFER_SIZE: usize = 1024;
const PAYLOAD_CAPACITY: usize = 256;
const STATUS_INTERVAL_SECS: u64 = 60;

static ACTION_CHANNEL: Channel<NoopRawMutex, MqttAction, ACTION_QUEUE> = Channel::new();
static EVENT_CHANNEL: Channel<NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE> =
    Channel::new();

type ActionSender = Sender<'static, NoopRawMutex, MqttAction, ACTION_QUEUE>;
type EventSender = Sender<'static, NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE>;
type EventReceiver = Receiver<'static, NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE>;

/// Start the MQTT manager and helper tasks. Call this once the Wi-Fi stack is ready.
pub fn start(stack: Stack<'static>, spawner: &Spawner) {
    let broker_ip = parse_broker_ip();
    let broker_port = parse_broker_port();
    let connection_settings = ConnectionSettings::unauthenticated(MQTT_CLIENT_ID);
    let settings = build_manager_settings(broker_ip, broker_port);

    let event_sender = EVENT_CHANNEL.sender();
    let action_receiver = ACTION_CHANNEL.receiver();
    let event_receiver = EVENT_CHANNEL.receiver();
    let action_sender = ACTION_CHANNEL.sender();

    spawner
        .spawn(mqtt_manager_task(
            stack,
            connection_settings,
            settings,
            event_sender,
            action_receiver,
        ))
        .ok()
        .expect("failed to spawn mqtt manager");

    spawner
        .spawn(mqtt_event_task(event_receiver, action_sender, stack))
        .ok()
        .expect("failed to spawn mqtt event task");

    spawner
        .spawn(status_publisher_task(stack, ACTION_CHANNEL.sender()))
        .ok()
        .expect("failed to spawn mqtt status task");
}

/// Queue a telemetry publish for the given reading.
pub async fn publish_reading(reading: &Reading) {
    match build_telemetry_action(reading) {
        Ok(action) => ACTION_CHANNEL.sender().send(action).await,
        Err(err) => warn!("failed to serialize telemetry payload: {err:?}"),
    }
}

fn build_manager_settings(address: Ipv4Address, port: u16) -> Settings {
    let mut settings = Settings::new(address, port);
    settings.ping_interval = Duration::from_secs(15);
    settings.connection_event_max_interval = Duration::from_secs(60);
    settings.reconnection_delay = Duration::from_secs(5);
    settings.poll_interval = Duration::from_millis(10);
    settings.response_timeout = Duration::from_secs(5);
    settings.stabilisation_interval = Duration::from_secs(10);
    settings
}

fn parse_broker_ip() -> Ipv4Address {
    let mut octets = [0u8; 4];
    let mut count = 0;
    for part in MQTT_BROKER_IP.split('.') {
        if count == 4 {
            panic!("MQTT_BROKER_IP has too many parts");
        }
        octets[count] = part
            .parse::<u8>()
            .unwrap_or_else(|_| panic!("invalid MQTT_BROKER_IP: {MQTT_BROKER_IP}"));
        count += 1;
    }
    if count != 4 {
        panic!("MQTT_BROKER_IP must have four octets");
    }
    Ipv4Address::new(octets[0], octets[1], octets[2], octets[3])
}

fn parse_broker_port() -> u16 {
    MQTT_BROKER_PORT
        .parse::<u16>()
        .unwrap_or_else(|_| panic!("invalid MQTT_BROKER_PORT: {MQTT_BROKER_PORT}"))
}

fn build_telemetry_action(reading: &Reading) -> Result<MqttAction, fmt::Error> {
    let mut payload: String<PAYLOAD_CAPACITY> = String::new();
    write!(
        &mut payload,
        "{{\"temperature_c\":{:.2},\"humidity_pct\":{:.2}}}",
        reading.temperature, reading.humidity
    )?;

    Ok(MqttAction::new(
        MQTT_TOPIC_TELEMETRY,
        payload,
        QualityOfService::Qos1,
        false,
    ))
}

fn build_status_action(stack: Stack<'static>) -> Result<MqttAction, fmt::Error> {
    let mut payload: String<PAYLOAD_CAPACITY> = String::new();
    let online = stack.is_link_up();
    write!(&mut payload, "{{\"online\":{}", online)?;

    if let Some(rssi) = read_rssi_dbm() {
        write!(&mut payload, ",\"rssi_dbm\":{}", rssi)?;
    } else {
        push_literal(&mut payload, ",\"rssi_dbm\":null")?;
    }

    write!(&mut payload, ",\"uptime_s\":{}", Instant::now().as_secs())?;
    write!(&mut payload, ",\"firmware\":\"{}\"", FIRMWARE)?;

    match ipv4_string(stack) {
        Some(ip) => write!(&mut payload, ",\"ip\":\"{}\"", ip)?,
        None => push_literal(&mut payload, ",\"ip\":null")?,
    }

    payload.push('}').map_err(|_| fmt::Error)?;

    Ok(MqttAction::new(
        MQTT_TOPIC_STATUS,
        payload,
        QualityOfService::Qos1,
        true,
    ))
}

fn ipv4_string(stack: Stack<'static>) -> Option<String<32>> {
    let config = stack.config_v4()?;
    let addr = config.address.address();
    let octets = addr.octets();
    let mut buf: String<32> = String::new();
    write!(
        &mut buf,
        "{}.{}.{}.{}",
        octets[0], octets[1], octets[2], octets[3]
    )
    .ok()?;
    Some(buf)
}

fn push_literal<const N: usize>(buf: &mut String<N>, literal: &str) -> Result<(), fmt::Error> {
    buf.push_str(literal).map_err(|_| fmt::Error)
}

fn read_rssi_dbm() -> Option<i32> {
    if wifi::sta_state() != WifiStaState::Connected {
        return None;
    }
    let mut rssi: i32 = 0;
    let err = unsafe { esp_wifi_sys::include::esp_wifi_sta_get_rssi(&mut rssi) };
    if err == 0 { Some(rssi) } else { None }
}

async fn enqueue_status(sender: ActionSender, stack: Stack<'static>) {
    match build_status_action(stack) {
        Ok(action) => sender.send(action).await,
        Err(err) => warn!("failed to serialize status payload: {err:?}"),
    }
}

#[derive(Clone)]
struct MqttAction {
    topic: &'static str,
    payload: String<PAYLOAD_CAPACITY>,
    qos: QualityOfService,
    retain: bool,
}

impl MqttAction {
    fn new(
        topic: &'static str,
        payload: String<PAYLOAD_CAPACITY>,
        qos: QualityOfService,
        retain: bool,
    ) -> Self {
        Self {
            topic,
            payload,
            qos,
            retain,
        }
    }
}

#[allow(async_fn_in_trait)]
impl MqttOperations for MqttAction {
    async fn perform<'a, 'b, C>(
        &'b mut self,
        client: &mut C,
        _client_id: &'a str,
        _connection_id: ConnectionId,
        _is_retry: bool,
    ) -> Result<(), ClientError>
    where
        C: Client<'a>,
    {
        client
            .publish(self.topic, self.payload.as_bytes(), self.qos, self.retain)
            .await
    }
}

#[derive(Clone)]
struct NoopApplicationEvent;

impl<const P: usize> FromApplicationMessage<P> for NoopApplicationEvent {
    fn from_application_message(
        _message: &ApplicationMessage<P>,
    ) -> Result<Self, EventHandlerError> {
        Ok(NoopApplicationEvent)
    }
}

#[embassy_executor::task]
async fn mqtt_manager_task(
    stack: Stack<'static>,
    connection_settings: ConnectionSettings<'static>,
    settings: Settings,
    event_sender: EventSender,
    action_receiver: Receiver<'static, NoopRawMutex, MqttAction, ACTION_QUEUE>,
) -> ! {
    mqtt_manager::run::<MqttAction, NoopApplicationEvent, 0, MQTT_BUFFER_SIZE, ACTION_QUEUE>(
        stack,
        connection_settings,
        settings,
        event_sender,
        action_receiver,
    )
    .await
}

#[embassy_executor::task]
async fn mqtt_event_task(
    receiver: EventReceiver,
    action_sender: ActionSender,
    stack: Stack<'static>,
) -> ! {
    loop {
        match receiver.receive().await {
            MqttEvent::Connected { .. } => info!("MQTT connected"),
            MqttEvent::ConnectionStable { .. } => enqueue_status(action_sender, stack).await,
            MqttEvent::Disconnected { error, .. } => warn!("MQTT disconnected: {error:?}"),
            MqttEvent::ApplicationEvent { .. } => {
                warn!("received unexpected MQTT application event")
            }
            MqttEvent::SubscriptionGrantedBelowMaximumQos { .. }
            | MqttEvent::PublishedMessageHadNoMatchingSubscribers { .. }
            | MqttEvent::NoSubscriptionExisted { .. } => {}
        }
    }
}

#[embassy_executor::task]
async fn status_publisher_task(stack: Stack<'static>, action_sender: ActionSender) -> ! {
    loop {
        enqueue_status(action_sender, stack).await;
        Timer::after(Duration::from_secs(STATUS_INTERVAL_SECS)).await;
    }
}
