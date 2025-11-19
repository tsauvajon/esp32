#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use blinksy_esp::ClocklessRmtError;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::main;
use esp_hal::time::Duration;
use log::info;
use pir_motion_sensor::led_strip::{RmtControl, build_led_controller};

esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY: Duration = Duration::from_secs(3); // HOw long to initially light up before trusting the PIR
const LIGHT_DURATION: Duration = Duration::from_secs(6); // How much time will the LEDs stay on after movement is detected
const GRACE_PERIOD: Duration = Duration::from_secs(3); // Don't extend the light duration if movement is re-detected in the grace period
const STEP: Duration = Duration::from_millis(100); // How frequently to check for movement

const REQUIRED_CONSECUTIVE_DETECTIONS: u8 = 3;

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

    let delay = Delay::new();
    let led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);
    let mut driver = Driver { led_control, delay };
    driver.light_on().unwrap();

    info!("Delaying start - {STARTUP_DELAY}!");
    delay.delay(STARTUP_DELAY);

    driver.light_off();

    let mut triggers = 0;
    loop {
        if sensor_pin.is_high() {
            triggers += 1;
            // Avoid false positives, by requiring detection several times in a row
            if triggers >= REQUIRED_CONSECUTIVE_DETECTIONS {
                info!("Initial motion!");
                driver.keep_on_until_silence(&sensor_pin).unwrap();
                triggers = 0;
                continue;
            }

            info!("Trigger {triggers}");
        } else {
            driver.light_off();
            triggers = 0;
        }

        delay.delay(STEP);
    }
}

struct Driver<'a> {
    led_control: RmtControl<'a>,
    delay: Delay,
}

impl<'a> Driver<'a> {
    fn light_on(&mut self) -> Result<(), ClocklessRmtError> {
        self.led_control.set_brightness(0.05);
        let elapsed_in_ms = blinksy_esp::time::elapsed().as_millis();
        self.led_control.tick(elapsed_in_ms)
    }

    fn light_off(&mut self) {
        self.led_control.set_brightness(0.0);
    }

    fn keep_on_until_silence(&mut self, sensor_pin: &Input) -> Result<(), ClocklessRmtError> {
        self.light_on()?;

        self.delay.delay(GRACE_PERIOD);
        let mut time_remaining = LIGHT_DURATION
            .checked_sub(GRACE_PERIOD)
            // Grace period > light duration - grace period is already expired
            .unwrap_or(Duration::ZERO);
        loop {
            if sensor_pin.is_high() {
                info!("Motion reset!");
                time_remaining = LIGHT_DURATION;
            }

            if time_remaining.le(&Duration::ZERO) {
                info!("Clear");
                self.light_off();
                return Ok(());
            }

            self.delay.delay(STEP);
            time_remaining = time_remaining.checked_sub(STEP).unwrap_or(Duration::ZERO);
        }
    }
}
