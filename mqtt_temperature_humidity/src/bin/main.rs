#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver},
};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::info;
use portable_temp_display::temp_humidity;
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

    let frequency = Rate::from_khz(I2C_FAST_MODE_KHZ);
    let i2c = I2c::new(i2c_driver, I2cConfig::default().with_frequency(frequency))
        .unwrap()
        .with_scl(scl)
        .with_sda(sda)
        .into_async();
    spawner
        .spawn(temp_humidity::run(i2c, SENSOR_CHANNEL.dyn_sender()))
        .unwrap();

    loop {
        // TODO: send mqtt data
    }
}
