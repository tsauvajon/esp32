use std::{thread::sleep, time::Duration};

use esp_idf_svc::{
    hal::{
        gpio::{Output, Pin, PinDriver},
        prelude::Peripherals,
    },
    sys::EspError,
};
use segment_display::SegmentDisplay;

mod segment_display;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let seg_e = PinDriver::output(peripherals.pins.gpio26)?; // blue - schematic 7
    let seg_d = PinDriver::output(peripherals.pins.gpio18)?; // purple - schematic 6
    let pin5 = PinDriver::output(peripherals.pins.gpio19)?; // grey - schematic 5
    let seg_c = PinDriver::output(peripherals.pins.gpio23)?; // white - schematic 4
    let seg_g = PinDriver::output(peripherals.pins.gpio5)?; // black - schematic 3

    let digit4 = PinDriver::output(peripherals.pins.gpio22)?; // yellow - schematic 2
    let digit1 = PinDriver::output(peripherals.pins.gpio2)?; // blue - schematic 8

    let seg_a = PinDriver::output(peripherals.pins.gpio25)?; // green - schematic 9
    let seg_f = PinDriver::output(peripherals.pins.gpio32)?; // yellow - schematic 10
    let digit2 = PinDriver::output(peripherals.pins.gpio12)?; // orange - schematic 11
    let digit3 = PinDriver::output(peripherals.pins.gpio4)?; // red - schematic 12
    let seg_b = PinDriver::output(peripherals.pins.gpio0)?; // brown - schematic 13

    let mut pin5 = pin5;
    pin5.set_high()?;

    // int ASeg = 9
    // int BSeg = 13
    // int CSeg = 4
    // int DSeg = 6
    // int ESeg = 7
    // int FSeg = 10
    // int GSeg = 3
    // int a1 = 8
    // int a2 = 11
    // int a3 = 12
    // int a4 = 2

    log::info!("Hello, world!");

    let mut segment_display = SegmentDisplay::try_new(
        digit1, digit2, digit3, digit4, seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g,
    )?;

    loop {
        segment_display.display(1234)?;
        // blink(&mut led)?;
    }
}

fn _blink(led: &mut PinDriver<'_, impl Pin, Output>) -> Result<(), EspError> {
    led.set_high()?;
    sleep(Duration::from_millis(1000));

    led.set_low()?;
    sleep(Duration::from_millis(1000));

    Ok(())
}
