#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver},
};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::info;
use portable_temp_display::segment_display::SegmentDisplay;
use portable_temp_display::temp_humidity;
use sht31::Reading;

const SENSOR_CHANNEL_SIZE: usize = 4;
static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, Reading, SENSOR_CHANNEL_SIZE> =
    Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

// https://esp32.implrust.com/i2c/esp32-i2c.html => 100 or 400 kHz
// https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-reference/peripherals/i2c.html
const I2C_FAST_MODE_KHZ: u32 = 400;

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    // Embassy
    let timg0 = peripherals.TIMG0;
    let interrupt = peripherals.SW_INTERRUPT;

    // SHT31 Temperature Humidity
    let i2c_driver = peripherals.I2C0;
    let scl = peripherals.GPIO9;
    let sda = peripherals.GPIO8;

    // 3461BS Segment Display
    let digit1 = peripherals.GPIO7;
    let seg_a = peripherals.GPIO5;
    let seg_f = peripherals.GPIO6;
    let digit2 = peripherals.GPIO10;
    let digit3 = peripherals.GPIO20;
    let seg_b = peripherals.GPIO21;

    let digit4 = peripherals.GPIO2; // TODO: SOLDER!
    let seg_c = peripherals.GPIO1;
    let seg_g = peripherals.GPIO0;
    // not soldered: decimal point
    let seg_d = peripherals.GPIO3;
    let seg_e = peripherals.GPIO4;

    let timer_group = TimerGroup::new(timg0);
    let sw_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(interrupt);
    esp_rtos::start(timer_group.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let frequency = Rate::from_khz(I2C_FAST_MODE_KHZ);
    let i2c = I2c::new(i2c_driver, I2cConfig::default().with_frequency(frequency))
        .unwrap()
        .with_scl(scl)
        .with_sda(sda)
        .into_async();
    spawner
        .spawn(temp_humidity::run(i2c, SENSOR_CHANNEL.dyn_sender()))
        .unwrap();

    let mut segment_display = SegmentDisplay::new(
        digit1, digit2, digit3, digit4, seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g,
    );
    let receiver = SENSOR_CHANNEL.receiver();

    run_display_loop(&mut segment_display, receiver).await;
}

async fn run_display_loop<'p>(
    segment_display: &mut SegmentDisplay<'p>,
    mut receiver: Receiver<'static, CriticalSectionRawMutex, Reading, SENSOR_CHANNEL_SIZE>,
) -> ! {
    let mut number_to_display = format_reading(receiver.receive().await);

    loop {
        segment_display.display(number_to_display).await;

        if let Ok(reading) = receiver.try_receive() {
            number_to_display = format_reading(reading);
        }
    }
}

// E.g. 23°C and 41% humidity will be displayed as 2341
fn format_reading(reading: Reading) -> u16 {
    (reading.temperature as u16 * 100) + (reading.humidity as u16 % 100)
}
