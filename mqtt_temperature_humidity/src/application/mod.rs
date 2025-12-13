use core::fmt::{self, Write};

use embassy_net::{Ipv4Address, Stack};
use embassy_time::Instant;
use esp_radio::wifi::{self, WifiStaState};
use heapless::String;
use log::warn;
use mountain_mqtt::data::quality_of_service::QualityOfService;
use sht31::Reading;

use crate::infrastructure::mqtt::MqttHandle;

pub(crate) const PAYLOAD_CAPACITY: usize = 256;
pub(crate) const STATUS_INTERVAL_SECS: u64 = 60;

const MQTT_TOPIC_TELEMETRY: &str = env!("MQTT_TOPIC_TELEMETRY");
const MQTT_TOPIC_STATUS: &str = env!("MQTT_TOPIC_STATUS");
const FIRMWARE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

pub(crate) struct ApplicationAction {
    pub topic: &'static str,
    pub payload: String<PAYLOAD_CAPACITY>,
    pub qos: QualityOfService,
    pub retain: bool,
}

pub(crate) fn build_telemetry_action(reading: &Reading) -> Result<ApplicationAction, fmt::Error> {
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

pub(crate) fn build_status_action(stack: Stack<'static>) -> Result<ApplicationAction, fmt::Error> {
    let mut payload: String<PAYLOAD_CAPACITY> = String::new();
    let online = stack.is_link_up();
    write!(&mut payload, "{{\"online\":{}", online)?;

    if let Some(rssi) = read_rssi_dbm() {
        write!(&mut payload, r#","rssi_dbm":{rssi}"#)?;
    } else {
        push_literal(&mut payload, ",\"rssi_dbm\":null")?;
    }

    write!(&mut payload, ",\"uptime_s\":{}", Instant::now().as_secs())?;
    write!(&mut payload, ",\"firmware\":\"{FIRMWARE}\"")?;
    write!(&mut payload, ",\"mac\":\"{}\"", MacAddress(wifi::sta_mac()))?;

    if let Some(config) = stack.config_v4() {
        let ip = IpAddress(config.address.address());
        write!(&mut payload, r#","ip":"{ip}""#)?;
    } else {
        push_literal(&mut payload, ",\"ip\":null")?;
    }

    payload.push('}').map_err(|_| fmt::Error)?;

    Ok(ApplicationAction {
        topic: MQTT_TOPIC_STATUS,
        payload,
        qos: QualityOfService::Qos1,
        retain: true,
    })
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

struct IpAddress(Ipv4Address);

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = &self.0.octets();
        write!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

impl MqttHandle {
    pub async fn publish_reading(&self, reading: &Reading) -> Result<(), fmt::Error> {
        let action = build_telemetry_action(reading)
            .inspect_err(|err| warn!("failed to serialize telemetry payload: {err:?}"))?;
        self.publish(action).await
    }
}
