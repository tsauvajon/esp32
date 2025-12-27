#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::future::pending;
use core::time::Duration;

use embassy_executor::{Spawner, task};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::timer::timg::TimerGroup;
use log::{error, info};
use pir_motion_sensor::led_strip::build_led_controller;
use pir_motion_sensor::lighting::{Driver, EmbassySleeper, LedStrip, STEP};
use pir_motion_sensor::mk_static;
use pir_motion_sensor::motion_detection::{
    MotionDetector, MotionState, PirMotionSensor, SharedMotionDetector, monitor_motion,
};

esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY: Duration = Duration::from_secs(3);
static MOTION_STATE: MotionState = MotionState::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer_group = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timer_group.timer0);

    info!("Embassy initialized");

    let movement_detection_pin = peripherals.GPIO17;
    let led_strip_data_pin = peripherals.GPIO16;

    let sensor_pin = Input::new(
        movement_detection_pin,
        InputConfig::default().with_pull(Pull::Down),
    );
    let motion_sensor = mk_static!(PirMotionSensor<'static>, PirMotionSensor::new(sensor_pin));

    let led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);
    let lights = LedStrip::new(led_control);
    let driver = mk_static!(
        Driver<LedStrip<'static>, EmbassySleeper>,
        Driver::new(lights, EmbassySleeper)
    );

    spawner
        .spawn(monitor_motion(motion_sensor, &MOTION_STATE, STEP))
        .unwrap();

    spawner.spawn(lighting_task(driver, &MOTION_STATE)).unwrap();

    loop {
        pending::<()>().await;
    }
}

#[task]
async fn lighting_task(
    driver: &'static mut Driver<LedStrip<'static>, EmbassySleeper>,
    state: &'static MotionState,
) -> ! {
    let mut detector = SharedMotionDetector::new(state);

    info!("initial lightning to indicate power on");
    if let Err(err) = driver.light_on() {
        error!("toggle lights on at startup: {err:?}");
    }
    driver.delay_for(STARTUP_DELAY).await;
    info!("initial lightning off");
    if let Err(err) = driver.light_off() {
        error!("toggle lights off at startup: {err:?}");
    }

    loop {
        if detector.motion_detected() {
            info!("motion detected, lighting");
            if let Err(err) = driver.keep_on_until_silence(&mut detector).await {
                error!("lighting sequence: {err:?}");
            }
            continue;
        } else if let Err(err) = driver.light_off() {
            info!("no motion detected, turning off");
            error!("keep lights off: {err:?}");
        }

        driver.delay_for(STEP).await;
    }
}
