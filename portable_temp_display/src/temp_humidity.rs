use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    i2c::master::{Config as I2cConfig, I2c, Instance as I2cInstance},
    time::Rate,
};
use log::{error, info};
use sht31::{Reading, SHT31, TemperatureUnit, mode::Sht31Reader};

// https://esp32.implrust.com/i2c/esp32-i2c.html => 100 or 400 kHz
// https://docs.espressif.com/projects/esp-idf/en/stable/esp32c3/api-reference/peripherals/i2c.html
const I2C_FAST_MODE_KHZ: u32 = 400;

#[embassy_executor::task]
pub async fn run<M, const N: usize>(
    i2c_instance: impl I2cInstance + 'static,
    scl: impl PeripheralOutput<'static>,
    sda: impl PeripheralOutput<'static>,
    sender: Sender<'static, M, Reading, N>,
) -> !
where
    M: RawMutex + 'static,
{
    let delay = &mut Delay {};
    let frequency = Rate::from_khz(I2C_FAST_MODE_KHZ);
    let i2c = I2c::new(i2c_instance, I2cConfig::default().with_frequency(frequency))
        .unwrap()
        .with_scl(scl)
        .with_sda(sda);
    let mut sht = SHT31::new(i2c, Delay {}).with_unit(TemperatureUnit::Celsius);

    info!("Hello world!");
    loop {
        match sht.read() {
            Ok(
                reading @ Reading {
                    temperature,
                    humidity,
                },
            ) => {
                info!("{temperature:.1}°C / {humidity:.0}% humidity");
                sender.send(reading).await;
            }
            Err(err) => error!("Reading: {err}"),
        }

        delay.delay_ms(500).await;
    }
}
