use std::{thread::sleep, time::Duration};

use esp_idf_svc::hal::task::watchdog::{config::Config, TWDTDriver};
use esp_idf_svc::{
    hal::{
        gpio::{Output, Pin, PinDriver},
        prelude::Peripherals,
    },
    sys::EspError,
};

mod segment_display;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    log::info!("Hello, world!");

    let mut twdt_driver = TWDTDriver::new(peripherals.twdt, &Config::default()).unwrap();
    let mut sub = twdt_driver.watch_current_task().unwrap();

    loop {
        sleep(Duration::from_millis(54));
        // blink(&mut led)?;
        // _segment_display()?;
        sub.feed().unwrap();
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
