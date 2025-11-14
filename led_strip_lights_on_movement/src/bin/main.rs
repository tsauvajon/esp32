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
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::main;
use esp_hal::time::Duration;
use log::info;
use pir_motion_sensor::led_strip::build_led_controller;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY_SEC: u64 = 3;

const LIGHT_DURATION_MS: u64 = 6_000; // How much time will the LEDs stay on after movement is detected
const GRACE_PERIOD_MS: u64 = 3_000; // Don't extend the light duration if movement is re-detected in the grace period
const STEP_MS: u64 = 100; // How frequently to check for movement

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let movement_detection_pin = peripherals.GPIO33;
    let led_strip_data_pin = peripherals.GPIO16;

    let sensor_pin = Input::new(
        movement_detection_pin,
        InputConfig::default().with_pull(Pull::Down),
    );

    let mut led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);

    let step = Duration::from_millis(STEP_MS);
    let grace_period = Duration::from_millis(GRACE_PERIOD_MS);
    let delay = Delay::new();
    led_control.set_brightness(0.05);
    let elapsed_in_ms = blinksy_esp::time::elapsed().as_millis();
    led_control.tick(elapsed_in_ms).unwrap();

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
        led_control.set_brightness(0.05);
        let elapsed_in_ms = blinksy_esp::time::elapsed().as_millis();
        led_control.tick(elapsed_in_ms).unwrap();

        let mut ms_remaining = LIGHT_DURATION_MS;
        delay.delay(grace_period);
        ms_remaining -= GRACE_PERIOD_MS;
        loop {
            delay.delay(step);
            ms_remaining -= STEP_MS;

            if ms_remaining <= 0 {
                info!("Clear");
                led_control.set_brightness(0.0);
                let elapsed_in_ms = blinksy_esp::time::elapsed().as_millis();
                led_control.tick(elapsed_in_ms).unwrap();
                break;
            }

            if sensor_pin.is_high() {
                info!("Motion reset!");
                ms_remaining = LIGHT_DURATION_MS;
                continue;
            }
        }
    }
}
