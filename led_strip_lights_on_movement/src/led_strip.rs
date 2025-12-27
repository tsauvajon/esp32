use blinksy::{
    Control, ControlBuilder,
    driver::ClocklessDriver,
    layout::Layout1d,
    layout1d,
    leds::Ws2812,
    markers::{Blocking, Dim1d},
};
use blinksy_esp::{ClocklessRmt, ClocklessRmtBuilder};
use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    peripherals,
    rmt::{Channel, Rmt, Tx},
    time::Rate,
};

use crate::single_color::{SingleColor, SingleColorParams};

layout1d!(pub Layout, 300);

pub type RmtControl<'p> = Control<
    { Layout::PIXEL_COUNT },
    { Layout::PIXEL_COUNT * 3 },
    Dim1d,
    Blocking,
    Layout,
    SingleColor,
    ClocklessDriver<
        Ws2812,
        ClocklessRmt<
            { Layout::PIXEL_COUNT * 3 * 8 + 1 },
            Ws2812,
            Channel<'p, esp_hal::Blocking, Tx>,
        >,
    >,
>;

pub fn build_led_controller<'p>(
    rmt: peripherals::RMT<'p>,
    data_pin: impl PeripheralOutput<'p>,
) -> RmtControl<'p> {
    let ws2812_rmt_driver = {
        let rmt_clock_frequency = Rate::from_mhz(80);
        let rmt = Rmt::new(rmt, rmt_clock_frequency).unwrap();
        let rmt_channel = rmt.channel2;

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

    ControlBuilder::new_1d()
        .with_layout::<Layout, { Layout::PIXEL_COUNT }>()
        .with_pattern::<SingleColor>(SingleColorParams::WledWarmWhite)
        .with_driver(ws2812_rmt_driver)
        .with_frame_buffer_size::<{ Ws2812::frame_buffer_size(Layout::PIXEL_COUNT) }>()
        .build()
}
