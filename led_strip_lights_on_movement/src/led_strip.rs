use blinksy::{
    Control, ControlBuilder,
    driver::ClocklessDriver,
    layout::Layout1d,
    layout1d,
    leds::Ws2812,
    markers::{Blocking, Dim1d},
};
use blinksy_esp::{ClocklessRmt, ClocklessRmtBuilder, time::elapsed};
use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    peripherals::{self, Peripherals},
    rmt::{Channel, Rmt, Tx},
    time::Rate,
};
use log::info;

use crate::single_colour::{SingleColour, SingleColourParams};

layout1d!(pub Layout, 300);

pub type RmtControl<'p> = Control<
    { Layout::PIXEL_COUNT },
    { Layout::PIXEL_COUNT * 3 },
    Dim1d,
    Blocking,
    Layout,
    SingleColour,
    ClocklessDriver<
        Ws2812,
        ClocklessRmt<
            { Layout::PIXEL_COUNT * 3 * 8 + 1 },
            Ws2812,
            Channel<'p, esp_hal::Blocking, Tx>,
        >,
    >,
>;

pub fn _example_run(peripherals: Peripherals) -> ! {
    let mut led_control = build_led_controller(peripherals.RMT, peripherals.GPIO0);
    led_control.set_brightness(0.6);

    info!("Started");

    loop {
        let elapsed_in_ms = elapsed().as_millis();
        led_control.tick(elapsed_in_ms).unwrap();
    }
}

#[embassy_executor::task]
pub async fn _led_tick_task(led_control: &'static mut RmtControl<'static>) {
    let elapsed_in_ms = elapsed().as_millis();
    led_control.tick(elapsed_in_ms).unwrap();
}

pub fn build_led_controller<'p>(
    rmt: peripherals::RMT<'p>,
    data_pin: impl PeripheralOutput<'p>,
) -> RmtControl<'p> {
    let ws2812_rmt_driver = {
        let rmt_clock_frequency = Rate::from_mhz(80);
        let rmt = Rmt::new(rmt, rmt_clock_frequency).unwrap();
        let rmt_channel = rmt.channel2;

        // Create the driver using the ClocklessRmt builder."]
        blinksy::driver::ClocklessDriver::default()
            .with_led::<Ws2812>()
            .with_writer(
                ClocklessRmtBuilder::default()
                    .with_rmt_buffer_size::<{ Layout::PIXEL_COUNT * 3 * 8 + 1 }>()
                    .with_led::<Ws2812>()
                    .with_channel(rmt_channel)
                    .with_pin(data_pin)
                    .build(),
            )
    };

    let led_control = ControlBuilder::new_1d()
        .with_layout::<Layout, { Layout::PIXEL_COUNT }>()
        .with_pattern::<SingleColour>(SingleColourParams::WarmWhite)
        .with_driver(ws2812_rmt_driver)
        .with_frame_buffer_size::<{ Ws2812::frame_buffer_size(Layout::PIXEL_COUNT) }>()
        .build();

    led_control
}
