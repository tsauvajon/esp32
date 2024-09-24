use std::{thread::sleep, time::Duration};

use esp_idf_svc::{
    hal::{gpio::PinDriver, prelude::Peripherals},
    sys::EspError,
};

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio5)?;

    log::info!("Hello, world!");

    loop {
        led.set_high()?;
        sleep(Duration::from_millis(1000));

        led.set_low()?;
        sleep(Duration::from_millis(1000));
    }
}
