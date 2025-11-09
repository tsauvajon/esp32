#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::{Duration, Rate};
use log::{error, info};
use sht31::mode::Sht31Reader;
use sht31::{SHT31, TemperatureUnit};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    run(peripherals);

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

fn run(peripherals: Peripherals) -> ! {
    let delay = Delay::new();
    // https://esp32.implrust.com/i2c/esp32-i2c.html => 100 or 400 kHz
    let frequency = Rate::from_khz(400);
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(frequency),
    )
    .unwrap()
    .with_scl(peripherals.GPIO22)
    .with_sda(peripherals.GPIO21);
    let mut sht = SHT31::new(i2c, delay).with_unit(TemperatureUnit::Celsius);

    info!("Hello world!");
    loop {
        match sht.read() {
            Ok(data) => info!("Data: {data:?}"),
            Err(err) => error!("Reading: {err}"),
        }

        delay.delay(Duration::from_millis(500));
    }
}
