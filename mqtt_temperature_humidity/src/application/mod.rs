use core::fmt::{self, Write};
use core::mem::MaybeUninit;
use core::result::Result as CoreResult;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_net::{Ipv4Address, Stack};
use embassy_time::Instant;
use esp_alloc::HEAP;
use esp_hal::rtc_cntl::SocResetReason;
use esp_hal::system;
use esp_radio::wifi::{self, WifiStaState};
use log::{info, warn};
use mountain_mqtt::data::quality_of_service::QualityOfService;
use serde::Serialize;
use serde::Serializer;
use serde_json_core::{heapless::String, ser::Error as SerdeJsonError, to_string};
use sht31::Reading;

pub const PAYLOAD_CAPACITY: usize = 256;

const MQTT_TOPIC_TELEMETRY: &str = env!("MQTT_TOPIC_TELEMETRY");
const MQTT_TOPIC_STATUS: &str = env!("MQTT_TOPIC_STATUS");
const FIRMWARE: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
static LAST_SENSOR_READING_SECS: AtomicU32 = AtomicU32::new(0);

pub struct Message {
    pub topic: &'static str,
    pub payload: String<PAYLOAD_CAPACITY>,
    pub qos: QualityOfService,
    pub retain: bool,
}

pub type MessageResult<T> = CoreResult<T, MessageError>;

#[derive(Debug)]
pub enum MessageError {
    Serialization(SerdeJsonError),
}

impl From<SerdeJsonError> for MessageError {
    fn from(value: SerdeJsonError) -> Self {
        Self::Serialization(value)
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::Serialization(err) => write!(f, "serialization error: {err:?}"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct StatusReporter;

#[allow(async_fn_in_trait)]
pub trait Publisher {
    async fn publish(&self, message: Message) -> MessageResult<()>;
}

pub trait StatusProvider: Send + Sync {
    fn build_status_message(&self, stack: Stack<'static>) -> MessageResult<Message>;
}

impl StatusProvider for StatusReporter {
    fn build_status_message(&self, stack: Stack<'static>) -> MessageResult<Message> {
        build_status_message(stack)
    }
}

pub async fn publish_reading<P>(publisher: &P, reading: &Reading) -> MessageResult<()>
where
    P: Publisher,
{
    let message = build_telemetry_message(reading)
        .inspect_err(|err| warn!("failed to serialize telemetry payload: {err:?}"))?;
    let topic = message.topic;
    publisher.publish(message).await?;
    info!(
        "published telemetry reading on {topic}: {:.1}°C / {:.0}% RH",
        reading.temperature, reading.humidity
    );
    Ok(())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    heap_total_bytes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heap_used_bytes: Option<u32>,
    #[serde(rename = "bssid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    connected_bssid: Option<MacAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gateway_ip: Option<Ipv4Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_reason: Option<ResetReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_sensor_ok_seconds_ago: Option<u64>,
}

pub fn build_telemetry_message(reading: &Reading) -> MessageResult<Message> {
    let payload = to_string::<_, PAYLOAD_CAPACITY>(&TelemetryPayload::from(reading))?;

    Ok(Message {
        topic: MQTT_TOPIC_TELEMETRY,
        payload,
        qos: QualityOfService::Qos1,
        retain: false,
    })
}

fn build_status_message(stack: Stack<'static>) -> MessageResult<Message> {
    let (heap_total_bytes, heap_used_bytes) = heap_usage_bytes();
    let now_secs = Instant::now().as_secs();
    let last_sensor_ok_seconds_ago = last_sensor_reading_age_secs(now_secs);
    let connected_ap = connected_ap_details();
    let (ip_address, gateway_ip) = match stack.config_v4() {
        Some(cfg) => (Some(cfg.address.address()), cfg.gateway),
        None => (None, None),
    };
    let status = StatusPayload {
        online: stack.is_link_up(),
        rssi_dbm: read_rssi_dbm(),
        uptime_seconds: now_secs,
        firmware: FIRMWARE,
        mac_address: MacAddress(wifi::sta_mac()),
        ip_address,
        gateway_ip,
        heap_total_bytes,
        heap_used_bytes,
        connected_bssid: connected_ap
            .as_ref()
            .map(|details| MacAddress::from(details.bssid)),
        channel: connected_ap.as_ref().map(|details| details.channel),
        reset_reason: system::reset_reason().map(ResetReason),
        last_sensor_ok_seconds_ago,
    };

    let payload = to_string::<_, PAYLOAD_CAPACITY>(&status)?;

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

fn heap_usage_bytes() -> (Option<u32>, Option<u32>) {
    let stats = HEAP.stats();
    let total = u32::try_from(stats.size).ok();
    let used = u32::try_from(stats.current_usage).ok();
    (total, used)
}

struct ConnectedApDetails {
    bssid: [u8; 6],
    channel: u8,
}

fn connected_ap_details() -> Option<ConnectedApDetails> {
    let mut record = MaybeUninit::<esp_wifi_sys::include::wifi_ap_record_t>::uninit();
    let err = unsafe { esp_wifi_sys::include::esp_wifi_sta_get_ap_info(record.as_mut_ptr()) };
    if err != 0 {
        return None;
    }
    let record = unsafe { record.assume_init() };
    Some(ConnectedApDetails {
        bssid: record.bssid,
        channel: record.primary as u8,
    })
}

pub fn record_sensor_reading(timestamp_secs: u64) {
    let clamped = timestamp_secs.min(u32::MAX as u64) as u32;
    LAST_SENSOR_READING_SECS.store(clamped, Ordering::Relaxed);
}

fn last_sensor_reading_age_secs(now_secs: u64) -> Option<u64> {
    let recorded = LAST_SENSOR_READING_SECS.load(Ordering::Relaxed) as u64;
    if recorded == 0 {
        None
    } else {
        Some(now_secs.saturating_sub(recorded))
    }
}

struct ResetReason(SocResetReason);

impl Serialize for ResetReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let label = match self.0 {
            SocResetReason::ChipPowerOn => "chip_power_on",
            SocResetReason::CoreSw => "core_sw",
            SocResetReason::CoreDeepSleep => "core_deep_sleep",
            SocResetReason::CoreMwdt0 => "core_mwdt0",
            SocResetReason::CoreMwdt1 => "core_mwdt1",
            SocResetReason::CoreRtcWdt => "core_rtc_wdt",
            SocResetReason::Cpu0Mwdt0 => "cpu0_mwdt0",
            SocResetReason::Cpu0Sw => "cpu0_sw",
            SocResetReason::Cpu0RtcWdt => "cpu0_rtc_wdt",
            SocResetReason::SysBrownOut => "sys_brown_out",
            SocResetReason::SysRtcWdt => "sys_rtc_wdt",
            SocResetReason::Cpu0Mwdt1 => "cpu0_mwdt1",
            SocResetReason::SysSuperWdt => "sys_super_wdt",
            SocResetReason::SysClkGlitch => "sys_clk_glitch",
            SocResetReason::CoreEfuseCrc => "core_efuse_crc",
            SocResetReason::CoreUsbUart => "core_usb_uart",
            SocResetReason::CoreUsbJtag => "core_usb_jtag",
            SocResetReason::CorePwrGlitch => "core_pwr_glitch",
        };
        serializer.serialize_str(label)
    }
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

impl From<[u8; 6]> for MacAddress {
    fn from(value: [u8; 6]) -> Self {
        Self(value)
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
