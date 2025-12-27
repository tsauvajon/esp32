#![no_std]

extern crate esp_backtrace as _;

pub mod led_strip;
pub mod lighting;
pub mod motion_detection;
pub mod single_color;

#[macro_export]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
