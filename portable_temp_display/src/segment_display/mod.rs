//! module segment_display
//! Supports Miuzei 4 digits 7-segments display

use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

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
        digit1: impl OutputPin + 'p,
        digit2: impl OutputPin + 'p,
        digit3: impl OutputPin + 'p,
        digit4: impl OutputPin + 'p,
        seg_a: impl OutputPin + 'p,
        seg_b: impl OutputPin + 'p,
        seg_c: impl OutputPin + 'p,
        seg_d: impl OutputPin + 'p,
        seg_e: impl OutputPin + 'p,
        seg_f: impl OutputPin + 'p,
        seg_g: impl OutputPin + 'p,
    ) -> Self {
        let mut segment_display = Self {
            digit1: Output::new(digit1, Level::Low, OutputConfig::default()),
            digit2: Output::new(digit2, Level::Low, OutputConfig::default()),
            digit3: Output::new(digit3, Level::Low, OutputConfig::default()),
            digit4: Output::new(digit4, Level::Low, OutputConfig::default()),
            seg_a: Output::new(seg_a, Level::Low, OutputConfig::default()),
            seg_b: Output::new(seg_b, Level::Low, OutputConfig::default()),
            seg_c: Output::new(seg_c, Level::Low, OutputConfig::default()),
            seg_d: Output::new(seg_d, Level::Low, OutputConfig::default()),
            seg_e: Output::new(seg_e, Level::Low, OutputConfig::default()),
            seg_f: Output::new(seg_f, Level::Low, OutputConfig::default()),
            seg_g: Output::new(seg_g, Level::Low, OutputConfig::default()),
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
