use cc1101::RadioMode;
use esp_hal::peripherals::Peripherals;
use log::{error, info};

use crate::setup_rx_tx;

pub fn run(peripherals: Peripherals) -> ! {
    let mut radio = setup_rx_tx(peripherals);

    info!("Started the receiver");

    let mut addr: u8 = 0;
    let mut buffer = [0u8; 64];

    loop {
        info!("Switching to RX...");
        radio.set_radio_mode(RadioMode::Receive).unwrap();
        info!("Now in RX mode");

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
