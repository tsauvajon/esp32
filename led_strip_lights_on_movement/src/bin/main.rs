#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::RefCell;
use core::future::pending;
use core::time::Duration;

use critical_section::Mutex;
use embassy_executor::{Spawner, task};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Event, Input, InputConfig, Io, Pull};
use esp_hal::timer::timg::TimerGroup;
use log::{error, info};
use pir_motion_sensor::led_strip::build_led_controller;
use pir_motion_sensor::lighting::{Driver, EmbassySleeper, LedStrip, MOTION_CHECK_STEP};
use pir_motion_sensor::mk_static;
use pir_motion_sensor::motion_detection::{MotionDetector, MotionState, SharedMotionDetector};

esp_bootloader_esp_idf::esp_app_desc!();

const STARTUP_DELAY: Duration = Duration::from_secs(3);
static MOTION_STATE: MotionState = MotionState::new();
static PIR_SENSOR: Mutex<RefCell<Option<Input<'static>>>> = Mutex::new(RefCell::new(None));

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(gpio_interrupt_handler);

    let timer_group = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timer_group.timer0);

    info!("Embassy initialized");

    let movement_detection_pin = peripherals.GPIO17;
    let led_strip_data_pin = peripherals.GPIO16;

    let sensor_pin = Input::new(
        movement_detection_pin,
        InputConfig::default().with_pull(Pull::Down),
    );
    critical_section::with(|cs| {
        let mut slot = PIR_SENSOR.borrow_ref_mut(cs);
        slot.replace(sensor_pin);
        if let Some(pin) = slot.as_mut() {
            MOTION_STATE.update(pin.is_high());
            pin.clear_interrupt();
            pin.listen(Event::AnyEdge);
        }
    });

    let led_control = build_led_controller(peripherals.RMT, led_strip_data_pin);
    let lights = LedStrip::new(led_control);
    let driver = mk_static!(
        Driver<LedStrip<'static>, EmbassySleeper>,
        Driver::new(lights, EmbassySleeper)
    );

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

        driver.delay_for(MOTION_CHECK_STEP).await;
    }
}

#[esp_hal::handler]
fn gpio_interrupt_handler() {
    critical_section::with(|cs| {
        let mut slot = PIR_SENSOR.borrow_ref_mut(cs);
        let Some(pin) = slot.as_mut() else {
            return;
        };

        if !pin.is_interrupt_set() {
            return;
        }

        MOTION_STATE.update(pin.is_high());
        pin.clear_interrupt();
    });
}
