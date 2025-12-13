#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use esp_radio::Controller;
use esp_radio::wifi::Config as WifiConfig;
use log::info;
use mqtt_sht31::mk_static;
use mqtt_sht31::mqtt;
use mqtt_sht31::temp_humidity;
use mqtt_sht31::wifi::start_wifi;
use sht31::Reading;

const SENSOR_CHANNEL_SIZE: usize = 4;
static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, Reading, SENSOR_CHANNEL_SIZE> =
    Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

// https://esp32.implrust.com/i2c/esp32-i2c.html => 100 or 400 kHz
// https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-reference/peripherals/i2c.html
const I2C_FAST_MODE_KHZ: u32 = 400;

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    // Embassy
    let timg0 = peripherals.TIMG0;
    let interrupt = peripherals.SW_INTERRUPT;

    // SHT31 Temperature Humidity
    let i2c_driver = peripherals.I2C0;
    let scl = peripherals.GPIO9;
    let sda = peripherals.GPIO8;

    let timer_group = TimerGroup::new(timg0);
    let sw_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(interrupt);
    esp_rtos::start(timer_group.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    // ######### SHT31 Temperature Sensor
    let frequency = Rate::from_khz(I2C_FAST_MODE_KHZ);
    let i2c = I2c::new(i2c_driver, I2cConfig::default().with_frequency(frequency))
        .unwrap()
        .with_scl(scl)
        .with_sda(sda)
        .into_async();
    spawner
        .spawn(temp_humidity::run(i2c, SENSOR_CHANNEL.dyn_sender()))
        .unwrap();
    let receiver = SENSOR_CHANNEL.receiver();

    // ######### WiFi/MQTT
    let rng = esp_hal::rng::Rng::new();
    let radio_init = &*mk_static!(Controller, esp_radio::init().unwrap());
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, WifiConfig::default()).unwrap();
    let stack = start_wifi(wifi_controller, interfaces, rng, &spawner).await;
    mqtt::start(stack, &spawner);

    loop {
        let reading = receiver.receive().await;
        println!("{reading:?}");
        let _ = mqtt::publish_reading(&reading).await;
    }
}
