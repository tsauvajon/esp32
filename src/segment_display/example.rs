use esp_idf_svc::{
    hal::{gpio::PinDriver, prelude::Peripherals},
    sys::EspError,
};

use crate::segment_display::SegmentDisplay;

pub fn _display_1234(peripherals: Peripherals) -> Result<(), EspError> {
    let seg_e = PinDriver::output(peripherals.pins.gpio26)?; // blue - schematic 7
    let seg_d = PinDriver::output(peripherals.pins.gpio18)?; // purple - schematic 6
    let decimal = PinDriver::output(peripherals.pins.gpio19)?; // grey - schematic 5
    let seg_c = PinDriver::output(peripherals.pins.gpio23)?; // white - schematic 4
    let seg_g = PinDriver::output(peripherals.pins.gpio5)?; // black - schematic 3
    let digit4 = PinDriver::output(peripherals.pins.gpio22)?; // yellow - schematic 2

    let digit1 = PinDriver::output(peripherals.pins.gpio2)?; // blue - schematic 8
    let seg_a = PinDriver::output(peripherals.pins.gpio25)?; // green - schematic 9
    let seg_f = PinDriver::output(peripherals.pins.gpio32)?; // yellow - schematic 10
    let digit2 = PinDriver::output(peripherals.pins.gpio12)?; // orange - schematic 11
    let digit3 = PinDriver::output(peripherals.pins.gpio4)?; // red - schematic 12
    let seg_b = PinDriver::output(peripherals.pins.gpio0)?; // brown - schematic 13

    let mut decimal = decimal;
    decimal.set_low()?;

    let mut segment_display = SegmentDisplay::try_new(
        digit1, digit2, digit3, digit4, seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g,
    )?;

    log::info!("Connected display");

    segment_display.display(1234)?;

    Ok(())
}
