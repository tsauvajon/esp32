#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::peripherals::Peripherals;
use ultrasound::notes::play_pink_panther_theme;
use ultrasound::run;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    run(peripherals);
}

fn _buzzer_demo(peripherals: Peripherals) {
    play_pink_panther_theme(peripherals);
    loop {
        esp_hal::delay::Delay::new().delay_millis(99999);
    }
}
