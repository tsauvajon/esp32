use core::fmt;

use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Stack};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use heapless::String;
use log::{info, warn};
use mountain_mqtt::client::{Client, ClientError, ConnectionSettings, EventHandlerError};
use mountain_mqtt::data::quality_of_service::QualityOfService;
use mountain_mqtt::mqtt_manager::{ConnectionId, MqttOperations};
use mountain_mqtt::packets::publish::ApplicationMessage;
use mountain_mqtt_embassy::mqtt_manager::{self, FromApplicationMessage, MqttEvent, Settings};
use sht31::Reading;

use crate::application::{
    ApplicationAction, PAYLOAD_CAPACITY, STATUS_INTERVAL_SECS, build_status_action,
    build_telemetry_action,
};

const MQTT_BROKER_IP: &str = env!("MQTT_BROKER_IP");
const MQTT_BROKER_PORT: &str = env!("MQTT_BROKER_PORT");
const MQTT_CLIENT_ID: &str = env!("MQTT_CLIENT_ID");
const MQTT_USERNAME: Option<&str> = option_env!("MQTT_USERNAME");
const MQTT_PASSWORD: Option<&str> = option_env!("MQTT_PASSWORD");

const ACTION_QUEUE: usize = 8;
const EVENT_QUEUE: usize = 8;
const MQTT_BUFFER_SIZE: usize = 1024;

type ActionChannel = Channel<NoopRawMutex, MqttAction, ACTION_QUEUE>;
type EventChannelInner = Channel<NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE>;

static ACTION_CHANNEL: static_cell::StaticCell<ActionChannel> = static_cell::StaticCell::new();
static EVENT_CHANNEL: static_cell::StaticCell<EventChannelInner> = static_cell::StaticCell::new();

type ActionSender = Sender<'static, NoopRawMutex, MqttAction, ACTION_QUEUE>;
type EventSender = Sender<'static, NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE>;
type EventReceiver = Receiver<'static, NoopRawMutex, MqttEvent<NoopApplicationEvent>, EVENT_QUEUE>;

#[derive(Clone)]
pub struct MqttHandle {
    action_sender: ActionSender,
}

pub fn start(stack: Stack<'static>, spawner: &Spawner) -> MqttHandle {
    let broker_ip = MQTT_BROKER_IP
        .parse()
        .unwrap_or_else(|_| panic!("invalid MQTT_BROKER_IP: {MQTT_BROKER_IP}"));
    let broker_port = MQTT_BROKER_PORT
        .parse()
        .unwrap_or_else(|_| panic!("invalid MQTT_BROKER_PORT: {MQTT_BROKER_PORT}"));
    let connection_settings =
        if let (Some(username), Some(password)) = (MQTT_USERNAME, MQTT_PASSWORD) {
            ConnectionSettings::authenticated(MQTT_CLIENT_ID, username, password.as_bytes())
        } else {
            ConnectionSettings::unauthenticated(MQTT_CLIENT_ID)
        };

    let settings = build_manager_settings(broker_ip, broker_port);

    let action_channel = ACTION_CHANNEL.init(ActionChannel::new());
    let event_channel = EVENT_CHANNEL.init(EventChannelInner::new());

    let event_sender = event_channel.sender();
    let action_receiver = action_channel.receiver();
    let action_sender = action_channel.sender();
    let event_receiver = event_channel.receiver();

    spawner
        .spawn(mqtt_manager_task(
            stack,
            connection_settings,
            settings,
            event_sender,
            action_receiver,
        ))
        .ok()
        .expect("spawn MQTT manager");

    spawner
        .spawn(mqtt_event_task(
            event_receiver,
            action_sender.clone(),
            stack,
        ))
        .ok()
        .expect("spawn MQTT event task");

    spawner
        .spawn(status_publisher_task(stack, action_sender.clone()))
        .ok()
        .expect("spawn MQTT status task");

    MqttHandle { action_sender }
}

pub async fn publish_reading(handle: &MqttHandle, reading: &Reading) -> Result<(), fmt::Error> {
    let action = build_telemetry_action(reading)
        .inspect_err(|err| warn!("failed to serialize telemetry payload: {err:?}"))?;
    handle.action_sender.send(action.into()).await;
    Ok(())
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

async fn enqueue_status(sender: ActionSender, stack: Stack<'static>) {
    match build_status_action(stack) {
        Ok(action) => sender.send(action.into()).await,
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

impl From<ApplicationAction> for MqttAction {
    fn from(value: ApplicationAction) -> Self {
        Self::new(value.topic, value.payload, value.qos, value.retain)
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
