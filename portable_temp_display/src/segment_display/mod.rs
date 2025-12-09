//! module segment_display
//! Supports Miuzei 4 digits 7-segments display

use esp_hal::gpio::Output;

pub mod digits;
pub mod numbers;

pub struct SegmentDisplay<'p> {
    pub(crate) digit1: Output<'p>,
    pub(crate) digit2: Output<'p>,
    pub(crate) digit3: Output<'p>,
    pub(crate) digit4: Output<'p>,
    pub(crate) seg_a: Output<'p>,
    pub(crate) seg_b: Output<'p>,
    pub(crate) seg_c: Output<'p>,
    pub(crate) seg_d: Output<'p>,
    pub(crate) seg_e: Output<'p>,
    pub(crate) seg_f: Output<'p>,
    pub(crate) seg_g: Output<'p>,
}

impl<'p> SegmentDisplay<'p> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        digit1: Output<'p>,
        digit2: Output<'p>,
        digit3: Output<'p>,
        digit4: Output<'p>,
        seg_a: Output<'p>,
        seg_b: Output<'p>,
        seg_c: Output<'p>,
        seg_d: Output<'p>,
        seg_e: Output<'p>,
        seg_f: Output<'p>,
        seg_g: Output<'p>,
    ) -> Self {
        let mut segment_display = Self {
            digit1,
            digit2,
            digit3,
            digit4,
            seg_a,
            seg_b,
            seg_c,
            seg_d,
            seg_e,
            seg_f,
            seg_g,
        };
        segment_display.clear();
        segment_display
    }

    pub fn clear(&mut self) {
        self.digit1.set_low();
        self.digit2.set_low();
        self.digit3.set_low();
        self.digit4.set_low();

        self.seg_a.set_low();
        self.seg_b.set_low();
        self.seg_c.set_low();
        self.seg_d.set_low();
        self.seg_e.set_low();
        self.seg_f.set_low();
        self.seg_g.set_low();
    }
}
