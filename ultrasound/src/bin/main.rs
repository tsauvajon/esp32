#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::Peripherals;
use esp_hal::timer::timg::TimerGroup;
use ultrasound::notes::{pink_panther, play_song};
use ultrasound::run;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(unsafe { peripherals.TIMG0.clone_unchecked() });
    esp_rtos::start(timg0.timer0);

    // TODO: Spawn some tasks
    let _ = spawner;

    run(peripherals);
}

fn _buzzer_demo(peripherals: Peripherals) {
    play_song(peripherals, pink_panther::TEMPO, &pink_panther::MELODY);
    loop {
        esp_hal::delay::Delay::new().delay_millis(99999);
    }
}
