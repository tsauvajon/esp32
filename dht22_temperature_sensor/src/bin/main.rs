#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_dht_rs::dht22::Dht22;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::peripherals::Peripherals;
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("Hello world!");
    humidity_and_temperature(peripherals);

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

fn humidity_and_temperature(peripherals: Peripherals) -> ! {
    // Flex = InputOutput
    let mut io = Flex::new(peripherals.GPIO32);
    io.apply_input_config(&InputConfig::default().with_pull(Pull::Up));
    io.apply_output_config(
        &OutputConfig::default()
            .with_drive_mode(DriveMode::PushPull)
            .with_pull(Pull::None),
    );
    io.set_input_enable(true);
    io.set_high();

    _test(&mut io);
}

fn _read_temp(io: &mut Flex) -> ! {
    let mut delay = Delay::new();
    let mut dht22 = Dht22::new(io, &mut delay);

    loop {
        Delay::new().delay_millis(5_000);
        match dht22.read() {
            Ok(sensor_reading) => log::info!(
                "DHT 22 Sensor - Temperature: {} °C, humidity: {} %",
                sensor_reading.temperature,
                sensor_reading.humidity
            ),
            Err(error) => log::error!("An error occurred while trying to read sensor: {:?}", error),
        }
    }
}

fn _test(io: &mut Flex) -> ! {
    Delay::new().delay_millis(5_000);
    log::info!("Starting");

    io.set_low();
    Delay::new().delay_millis(18);
    io.set_high();
    Delay::new().delay_micros(48);

    loop {
        for _ in 0..20 {
            if io.is_high() {
                log::info!("GPIO32 HIGH");
            } else {
                log::info!("GPIO32 LOW");
            }
            Delay::new().delay_millis(1);
        }

        log::info!("---")
    }
}
