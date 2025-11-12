#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::Controller;
use esp_radio::wifi::Config as WifiConfig;
use log::info;
use webserver::mk_static;
use webserver::web::WebApp;
use webserver::wifi::start_wifi;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new();
    let radio_init = &*mk_static!(Controller, esp_radio::init().unwrap());

    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, WifiConfig::default()).unwrap();

    let stack = start_wifi(wifi_controller, interfaces, rng, &spawner).await;
    let web_app = WebApp::default();
    web_app.spawn_tasks(&spawner, stack);

    info!("Spawned web server tasks");

    loop {
        webserver::wifi::socket(stack).await;
        // Delay {}.delay_ms(u32::MAX).await;
    }
}
