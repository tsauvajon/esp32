use core::fmt::{self, Write};

use embassy_net::Stack;
use embassy_time::Instant;
use esp_radio::wifi::{self, WifiStaState};
use heapless::String;
use log::warn;
use mountain_mqtt::data::quality_of_service::QualityOfService;
use sht31::Reading;

pub const PAYLOAD_CAPACITY: usize = 256;

const MQTT_TOPIC_TELEMETRY: &str = env!("MQTT_TOPIC_TELEMETRY");
const MQTT_TOPIC_STATUS: &str = env!("MQTT_TOPIC_STATUS");
const FIRMWARE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

pub struct ApplicationAction {
    pub topic: &'static str,
    pub payload: String<PAYLOAD_CAPACITY>,
    pub qos: QualityOfService,
    pub retain: bool,
}

#[derive(Clone, Copy)]
pub struct StatusReporter;

#[allow(async_fn_in_trait)]
pub trait ApplicationPublisher {
    async fn publish(&self, action: ApplicationAction) -> Result<(), fmt::Error>;
}

pub trait StatusProvider: Send + Sync {
    fn build_status_action(&self, stack: Stack<'static>) -> Result<ApplicationAction, fmt::Error>;
}

impl StatusProvider for StatusReporter {
    fn build_status_action(&self, stack: Stack<'static>) -> Result<ApplicationAction, fmt::Error> {
        build_status_action(stack)
    }
}

pub async fn publish_reading<P>(publisher: &P, reading: &Reading) -> Result<(), fmt::Error>
where
    P: ApplicationPublisher,
{
    let action = build_telemetry_action(reading)
        .inspect_err(|err| warn!("failed to serialize telemetry payload: {err:?}"))?;
    publisher.publish(action).await
}

pub fn build_telemetry_action(reading: &Reading) -> Result<ApplicationAction, fmt::Error> {
    let mut payload: String<PAYLOAD_CAPACITY> = String::new();
    write!(
        &mut payload,
        "{{\"temperature_c\":{:.2},\"humidity_pct\":{:.2}}}",
        reading.temperature, reading.humidity
    )?;

    Ok(ApplicationAction {
        topic: MQTT_TOPIC_TELEMETRY,
        payload,
        qos: QualityOfService::Qos1,
        retain: false,
    })
}

fn build_status_action(stack: Stack<'static>) -> Result<ApplicationAction, fmt::Error> {
    let mut payload: String<PAYLOAD_CAPACITY> = String::new();
    let online = stack.is_link_up();
    write!(&mut payload, "{{\"online\":{}", online)?;

    if let Some(rssi) = read_rssi_dbm() {
        write!(&mut payload, r#","rssi_dbm":{rssi}"#)?;
    } else {
        payload
            .push_str(",\"rssi_dbm\":null")
            .map_err(|_| fmt::Error)?;
    }

    write!(&mut payload, ",\"uptime_s\":{}", Instant::now().as_secs())?;
    write!(&mut payload, ",\"firmware\":\"{FIRMWARE}\"")?;
    write!(&mut payload, ",\"mac\":\"{}\"", MacAddress(wifi::sta_mac()))?;

    if let Some(config) = stack.config_v4() {
        let ip = config.address.address();
        write!(&mut payload, r#","ip":"{ip}""#)?;
    } else {
        payload.push_str(",\"ip\":null").map_err(|_| fmt::Error)?;
    }

    payload.push('}').map_err(|_| fmt::Error)?;

    Ok(ApplicationAction {
        topic: MQTT_TOPIC_STATUS,
        payload,
        qos: QualityOfService::Qos1,
        retain: true,
    })
}

fn read_rssi_dbm() -> Option<i32> {
    if wifi::sta_state() != WifiStaState::Connected {
        return None;
    }
    let mut rssi: i32 = 0;
    let err = unsafe { esp_wifi_sys::include::esp_wifi_sta_get_rssi(&mut rssi) };
    if err == 0 { Some(rssi) } else { None }
}

struct MacAddress([u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = &self.0;
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        )
    }
}
