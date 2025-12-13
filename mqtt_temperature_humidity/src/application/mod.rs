use core::fmt::{self, Write};

use embassy_net::{Ipv4Address, Stack};
use embassy_time::Instant;
use esp_radio::wifi::{self, WifiStaState};
use log::warn;
use mountain_mqtt::data::quality_of_service::QualityOfService;
use serde::Serialize;
use serde::Serializer;
use serde_json_core::heapless::String;
use serde_json_core::to_string;
use sht31::Reading;

pub const PAYLOAD_CAPACITY: usize = 256;

const MQTT_TOPIC_TELEMETRY: &str = env!("MQTT_TOPIC_TELEMETRY");
const MQTT_TOPIC_STATUS: &str = env!("MQTT_TOPIC_STATUS");
const FIRMWARE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

pub struct Message {
    pub topic: &'static str,
    pub payload: String<PAYLOAD_CAPACITY>,
    pub qos: QualityOfService,
    pub retain: bool,
}

#[derive(Clone, Copy)]
pub struct StatusReporter;

#[allow(async_fn_in_trait)]
pub trait Publisher {
    async fn publish(&self, message: Message) -> Result<(), fmt::Error>;
}

pub trait StatusProvider: Send + Sync {
    fn build_status_message(&self, stack: Stack<'static>) -> Result<Message, fmt::Error>;
}

impl StatusProvider for StatusReporter {
    fn build_status_message(&self, stack: Stack<'static>) -> Result<Message, fmt::Error> {
        build_status_message(stack)
    }
}

pub async fn publish_reading<P>(publisher: &P, reading: &Reading) -> Result<(), fmt::Error>
where
    P: Publisher,
{
    let message = build_telemetry_message(reading)
        .inspect_err(|err| warn!("failed to serialize telemetry payload: {err:?}"))?;
    publisher.publish(message).await
}

#[derive(Serialize)]
struct TelemetryPayload {
    #[serde(rename = "temperature_c")]
    temperature_celsius: f32,
    #[serde(rename = "humidity_pct")]
    humidity_percent: f32,
}

impl From<&Reading> for TelemetryPayload {
    fn from(reading: &Reading) -> Self {
        Self {
            temperature_celsius: reading.temperature,
            humidity_percent: reading.humidity,
        }
    }
}

#[derive(Serialize)]
struct StatusPayload {
    online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    rssi_dbm: Option<i32>,
    #[serde(rename = "uptime_s")]
    uptime_seconds: u64,
    firmware: &'static str,
    #[serde(rename = "mac")]
    mac_address: MacAddress,
    #[serde(rename = "ip")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ip_address: Option<Ipv4Address>,
}

pub fn build_telemetry_message(reading: &Reading) -> Result<Message, fmt::Error> {
    let payload = to_string::<_, PAYLOAD_CAPACITY>(&TelemetryPayload::from(reading))
        .map_err(|_| fmt::Error)?;

    Ok(Message {
        topic: MQTT_TOPIC_TELEMETRY,
        payload,
        qos: QualityOfService::Qos1,
        retain: false,
    })
}

fn build_status_message(stack: Stack<'static>) -> Result<Message, fmt::Error> {
    let status = StatusPayload {
        online: stack.is_link_up(),
        rssi_dbm: read_rssi_dbm(),
        uptime_seconds: Instant::now().as_secs(),
        firmware: FIRMWARE,
        mac_address: MacAddress(wifi::sta_mac()),
        ip_address: stack.config_v4().map(|cfg| cfg.address.address()),
    };

    let payload = to_string::<_, PAYLOAD_CAPACITY>(&status).map_err(|_| fmt::Error)?;

    Ok(Message {
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

impl Serialize for MacAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = String::<17>::new();
        write!(&mut buf, "{self}").map_err(|_| serde::ser::Error::custom("format MAC address"))?;
        serializer.serialize_str(buf.as_str())
    }
}
