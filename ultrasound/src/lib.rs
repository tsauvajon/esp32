#![no_std]

use esp_hal::{
    peripherals::Peripherals,
    time::{Duration, Instant},
};
use log::info;

pub fn run(_peripherals: Peripherals) -> ! {
    loop {
        info!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
