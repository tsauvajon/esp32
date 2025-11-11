use cc1101::RadioMode;
use esp_hal::{delay::Delay, peripherals::Peripherals, time::Duration};
use log::info;

use crate::setup_rx_tx;

pub fn run(peripherals: Peripherals) -> ! {
    let mut radio = setup_rx_tx(peripherals);

    info!("Started the receiver");

    let mut _addr: u8 = 0;
    let mut _buffer = [0u8; 61];

    loop {
        info!("Switching to TX...");
        radio.set_radio_mode(RadioMode::Transmit).unwrap();
        info!("Now in TX mode");

        // TODO: implement sending data
        info!("unimplemented");

        Delay::new().delay(Duration::from_secs(60));
    }
}
