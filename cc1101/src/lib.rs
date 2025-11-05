#![no_std]

use cc1101::{Cc1101, RadioMode};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::master::{Config as SpiMasterConfig, Spi},
};
use log::{error, info};

pub fn run(peripherals: Peripherals) -> ! {
    let spi_bus = Spi::new(peripherals.SPI2, SpiMasterConfig::default())
        .unwrap()
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO23)
        .with_miso(peripherals.GPIO19);
    let chip_select = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new(spi_bus, chip_select, Delay::new()).unwrap();
    let mut radio = Cc1101::new(spi_device).unwrap();

    info!("Configured the pins - setting receive mode");
    radio.set_radio_mode(RadioMode::Receive).unwrap();

    let mut addr: u8 = 0;
    let mut buffer = [0u8; 64];
    info!("Started the receiver");

    loop {
        let len = match radio.receive(&mut addr, &mut buffer) {
            Ok(len) => len,
            Err(err) => {
                error!("Receive: {err:?}");
                continue;
            }
        };

        info!("Received {len} bytes from 0x{addr:02X}");
        info!("Payload: {:02X?}", &buffer[..len as usize]);
    }
}
