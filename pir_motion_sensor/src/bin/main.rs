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
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::time::Duration;
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY_SEC: u64 = 3;
const LIGHT_DURATION_MS: u64 = 6000;
const STEP_MS: u64 = 100;

#[main]
fn main() -> ! {
    // generator version: 1.0.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let sensor_pin = Input::new(
        peripherals.GPIO33,
        InputConfig::default().with_pull(Pull::Down),
    );

    let mut leds = [
        Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO12, Level::Low, OutputConfig::default()),
    ];

    let step = Duration::from_millis(STEP_MS);
    let delay = Delay::new();

    info!("Delaying start - {STARTUP_DELAY_SEC} seconds!");
    delay.delay(Duration::from_secs(STARTUP_DELAY_SEC));

    // Avoid false positives
    let mut triggers = 0;
    loop {
        if sensor_pin.is_low() {
            triggers = 0;
            delay.delay(step);
            continue;
        }

        if triggers < 3 {
            info!("Trigger {triggers}");
            triggers += 1;
            delay.delay(step);
            continue;
        }

        triggers = 0;
        info!("Initial motion!");
        for led in &mut leds {
            led.set_high();
        }

        let mut hundreds_of_ms_remaining = LIGHT_DURATION_MS;
        loop {
            delay.delay(step);
            hundreds_of_ms_remaining -= STEP_MS;

            if hundreds_of_ms_remaining <= 0 {
                info!("Clear");
                for led in &mut leds {
                    led.set_low();
                }
                break;
            }

            if sensor_pin.is_high() {
                info!("Motion reset!");
                hundreds_of_ms_remaining = LIGHT_DURATION_MS;
                continue;
            }
        }
    }
}
