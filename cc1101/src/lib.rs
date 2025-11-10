#![no_std]

use cc1101::{AddressFilter, AutoCalibration, Cc1101, ModulationFormat, PacketLength, SyncMode};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    Blocking,
    delay::Delay,
    gpio::{Level, Output, OutputConfig, Pull},
    peripherals::Peripherals,
    spi::{
        Mode,
        master::{Config as SpiMasterConfig, Spi},
    },
    time::Rate,
};
use log::info;

pub mod read_data;
pub mod write_data;

/// Common setup for transmitting and receiving
fn setup_rx_tx<'a>(
    peripherals: Peripherals,
) -> Cc1101<ExclusiveDevice<Spi<'a, Blocking>, Output<'a>, Delay>> {
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
    radio.reset_chip().unwrap();
    radio.set_defaults().unwrap();

    radio.set_frequency(433_000_000).unwrap();
    radio.set_data_rate(600).unwrap();
    radio
        .set_modulation_format(ModulationFormat::AmplitudeShiftOnOffKeying)
        .unwrap();
    radio.set_chanbw(58_000).unwrap();
    radio.set_deviation(400_000).unwrap();
    radio.set_sync_mode(SyncMode::Disabled).unwrap();
    radio.set_packet_length(PacketLength::Variable(61)).unwrap();
    radio
        .set_autocalibration(AutoCalibration::FromIdle)
        .unwrap();
    radio.set_address_filter(AddressFilter::Disabled).unwrap();
    radio.crc_enable(false).unwrap();
    radio.white_data_enable(false).unwrap();

    info!("Configured the radio rules");

    let (partnum, version) = radio.get_hw_info().unwrap();
    info!("CC1101 PARTNUM={partnum:#X}, VERSION={version:#X}");

    radio
}
