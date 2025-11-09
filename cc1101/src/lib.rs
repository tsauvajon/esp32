#![no_std]

use cc1101::{
    AddressFilter, Cc1101, Modulation, PacketLength, RadioMode, SyncMode,
    lowlevel::types::AutoCalibration,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig, Pull},
    peripherals::Peripherals,
    spi::{
        Mode,
        master::{Config as SpiMasterConfig, Spi},
    },
    time::Rate,
};
use log::{error, info};

pub fn run(peripherals: Peripherals) -> ! {
    info!("Delaying initial start (0.4 seconds)");
    let delay = Delay::new();
    delay.delay_millis(400);

    let mut spi_bus = Spi::new(
        peripherals.SPI3, // GPIO5, GPIO18, GPIO19, and GPIO21 to GPIO23
        SpiMasterConfig::default()
            // https://esp32.implrust.com/spi/esp32-spi.html
            .with_frequency(Rate::from_mhz(60))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_miso(peripherals.GPIO19)
    .with_mosi(peripherals.GPIO23);

    let mut data = [0x30 | 0x80, 0x00]; // SNOP | read
    spi_bus.transfer(&mut data).unwrap();
    info!("SNOP response: {data:X?}");

    let chip_select = Output::new(
        peripherals.GPIO5,
        Level::Low,
        OutputConfig::default().with_pull(Pull::Up),
    );
    let spi_device = ExclusiveDevice::new(spi_bus, chip_select, Delay::new()).unwrap();
    let mut radio = Cc1101::new(spi_device).unwrap();
    radio.reset().unwrap();

    radio.set_frequency(433_920_000).unwrap();
    radio.set_data_rate(38_383).unwrap();
    radio
        .set_modulation(Modulation::GaussianFrequencyShiftKeying)
        .unwrap();
    radio.set_chanbw(58_000).unwrap();
    radio.set_deviation(20_000).unwrap();
    radio.set_sync_mode(SyncMode::Disabled).unwrap();
    radio.set_packet_length(PacketLength::Variable(61)).unwrap();
    radio
        .set_autocalibration(AutoCalibration::FromIdle)
        .unwrap();
    radio.set_address_filter(AddressFilter::Disabled).unwrap();
    info!("Configured the radio rules");

    info!("Switching to RX...");
    radio.set_radio_mode(RadioMode::Receive).unwrap();
    info!("Now in RX mode");

    let (partnum, version) = radio.get_hw_info().unwrap();
    info!("CC1101 PARTNUM={partnum:#X}, VERSION={version:#X}");

    let mut addr: u8 = 0;
    let mut buffer = [0u8; 64];
    info!("Started the receiver");
    // radio.set_raw_mode().unwrap();

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
