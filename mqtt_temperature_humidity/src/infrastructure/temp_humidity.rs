use embassy_sync::channel::DynamicSender;
use embassy_time::{Delay, Instant};
use embedded_hal_async::delay::DelayNs;
use esp_hal::{Async, i2c::master::I2c};
use log::{error, info};
use sht31::{Reading, SHT31, TemperatureUnit, mode::Sht31Reader};

use crate::application;

#[embassy_executor::task]
pub async fn run(i2c: I2c<'static, Async>, sender: DynamicSender<'static, Reading>) -> ! {
    let delay = &mut Delay {};
    let mut sht = SHT31::new(i2c, Delay {}).with_unit(TemperatureUnit::Celsius);

    info!("Running the SHT31 temp sensor loop");
    loop {
        match sht.read() {
            Ok(
                reading @ Reading {
                    temperature,
                    humidity,
                },
            ) => {
                info!("sensor measured: {temperature:.1}°C / {humidity:.0}% humidity");
                sender.send(reading).await;
                application::record_sensor_reading(Instant::now().as_secs());
            }
            Err(err) => error!("Reading: {err}"),
        }

        delay.delay_ms(5_000).await;
    }
}
