#![no_std]

use cc1101::{
    Cc1101, Modulation, PacketLength, RadioMode, SyncMode, lowlevel::types::AutoCalibration,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::master::{Config as SpiMasterConfig, Spi},
};
use log::{error, info};

pub fn run(peripherals: Peripherals) -> ! {
    info!("Delaying initial start (2 seconds)");
    Delay::new().delay_millis(2000);

    let spi_bus = Spi::new(peripherals.SPI2, SpiMasterConfig::default())
        .unwrap()
        .with_sck(peripherals.GPIO18)
        .with_miso(peripherals.GPIO19)
        .with_mosi(peripherals.GPIO23);
    let chip_select = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new(spi_bus, chip_select, Delay::new()).unwrap();

    info!("Configured the pins - now configuring the radio");
    let mut radio = Cc1101::new(spi_device).unwrap();
    let (partnum, version) = radio.get_hw_info().unwrap();
    info!("CC1101 PARTNUM={:#X}, VERSION={:#X}", partnum, version);
    radio.set_frequency(433_920_000).unwrap();
    info!("Set frequence");
    radio.set_data_rate(38_383).unwrap();
    info!("Set data rate");
    radio
        .set_modulation(Modulation::GaussianFrequencyShiftKeying)
        .unwrap();
    info!("Set modulation");
    radio.set_chanbw(58_000).unwrap();
    info!("Set chanbw");
    radio.set_deviation(20_000).unwrap();
    info!("Set deviation");
    radio.set_sync_mode(SyncMode::MatchPartial(0xD391)).unwrap();
    info!("Set sync mode");
    radio.set_packet_length(PacketLength::Variable(61)).unwrap();
    info!("Set packet length");
    radio
        .set_autocalibration(AutoCalibration::FromIdle)
        .unwrap();
    info!("Set auto calibration");
    let (partnum, version) = radio.get_hw_info().unwrap();
    info!("CC1101 PARTNUM={:#X}, VERSION={:#X}", partnum, version);
    radio.set_radio_mode(RadioMode::Receive).unwrap();
    info!("Set receive mode");

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
