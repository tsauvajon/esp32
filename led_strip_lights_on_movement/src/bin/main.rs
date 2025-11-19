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

    const STEP: Duration = Duration::from_millis(STEP_MS);

    let delay = Delay::new();
    let led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);
    let mut driver = Driver { led_control, delay };
    driver.light_on().unwrap();

    info!("Delaying start - {STARTUP_DELAY_SEC} seconds!");
    delay.delay(Duration::from_secs(STARTUP_DELAY_SEC));

    driver.light_off();

    // Avoid false positives
    let mut triggers = 0;
    loop {
        if sensor_pin.is_low() {
            driver.light_off();
            triggers = 0;
            delay.delay(STEP);
            continue;
        }

        if triggers < 3 {
            info!("Trigger {triggers}");
            triggers += 1;
            delay.delay(STEP);
            continue;
        }

        triggers = 0;
        info!("Initial motion!");

        driver.keep_on_until_silence(&sensor_pin).unwrap();
    }
}

struct Driver<'a> {
    led_control: RmtControl<'a>,
    delay: Delay,
}

impl<'a> Driver<'a> {
    const STEP: Duration = Duration::from_millis(STEP_MS);
    const GRACE_PERIOD: Duration = Duration::from_millis(GRACE_PERIOD_MS);

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

        let mut ms_remaining = LIGHT_DURATION_MS;
        self.delay.delay(Self::GRACE_PERIOD);
        ms_remaining -= GRACE_PERIOD_MS;
        loop {
            self.delay.delay(Self::STEP);
            ms_remaining -= STEP_MS;

            if ms_remaining <= 0 {
                info!("Clear");
                self.light_off();
                return Ok(());
            }

            if sensor_pin.is_high() {
                info!("Motion reset!");
                ms_remaining = LIGHT_DURATION_MS;
            }
        }
    }
}
