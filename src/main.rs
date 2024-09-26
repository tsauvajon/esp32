use std::{thread::sleep, time::Duration};

use dht_sensor::dht11;
use dht_sensor::DhtReading;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::task::watchdog::{config::Config, TWDTDriver};
use esp_idf_svc::{
    hal::{
        gpio::{Output, Pin, PinDriver},
        prelude::Peripherals,
    },
    sys::EspError,
};
use log::{error, info};

mod segment_display;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    humidity_and_temperature()?;
    info!("Done!");

    loop {
        sleep(Duration::from_millis(54));
    }
}

fn _blink(led: &mut PinDriver<'_, impl Pin, Output>) -> Result<(), EspError> {
    led.set_high()?;
    sleep(Duration::from_millis(1000));

    led.set_low()?;
    sleep(Duration::from_millis(1000));

    Ok(())
}

pub fn _segment_display() -> Result<(), EspError> {
    let peripherals = Peripherals::take()?;
    segment_display::example::_display_1234(peripherals)
}

pub fn humidity_and_temperature() -> Result<(), EspError> {
    let peripherals = Peripherals::take().unwrap();
    let pin = peripherals.pins.gpio23;
    let mut sensor = PinDriver::input_output(pin).unwrap();
    sensor.set_high().unwrap();
    FreeRtos::delay_ms(1000);
    loop {
        match dht11::Reading::read(&mut delay::Delay::new(2_000_000), &mut sensor) {
            Ok(reading) => info!(
                "[Temperature: {}\tRelative humidity: {}",
                reading.temperature, reading.relative_humidity
            ),
            Err(err) => error!("Failed to retrieve information: {err:?}"),
        }
        FreeRtos::delay_ms(3000);
    }
}

fn _watchdog() {
    let peripherals = Peripherals::take().unwrap();
    let mut twdt_driver = TWDTDriver::new(peripherals.twdt, &Config::default()).unwrap();
    let mut sub = twdt_driver.watch_current_task().unwrap();
    loop {
        sub.feed().unwrap();
    }
}
