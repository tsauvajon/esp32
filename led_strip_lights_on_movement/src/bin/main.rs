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
use pir_motion_sensor::lighting::{Driver, LedStrip, STEP};
use pir_motion_sensor::motion_detection::{MotionDetector, PirMotionSensor};

esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY: Duration = Duration::from_secs(3); // How long to initially light up before trusting the PIR
const REQUIRED_CONSECUTIVE_DETECTIONS: u8 = 3;

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let movement_detection_pin = peripherals.GPIO17;
    let led_strip_data_pin = peripherals.GPIO16;

    let sensor_pin = Input::new(
        movement_detection_pin,
        InputConfig::default().with_pull(Pull::Down),
    );
    let mut motion_sensor = PirMotionSensor::new(sensor_pin);

    let delay = Delay::new();
    let led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);
    let lights = LedStrip::new(led_control);
    let mut driver = Driver::new(lights, delay);
    driver.light_on().unwrap();

    info!("Delaying start - {STARTUP_DELAY}!");
    driver.delay_for(STARTUP_DELAY);

    driver.light_off().unwrap();

    let mut triggers = 0;
    loop {
        if motion_sensor.motion_detected() {
            triggers += 1;
            // Avoid false positives, by requiring detection several times in a row
            if triggers >= REQUIRED_CONSECUTIVE_DETECTIONS {
                info!("Initial motion!");
                driver.keep_on_until_silence(&mut motion_sensor).unwrap();
                triggers = 0;
                continue;
            }

            info!("Trigger {triggers}");
        } else {
            driver.light_off().unwrap();
            triggers = 0;
        }

        driver.delay_for(STEP);
    }
}
