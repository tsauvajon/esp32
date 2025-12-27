//! Module single_color lights all LEDs in a strip with a single, unified color.

use blinksy::{color::Hsv, layout::Layout1d, markers::Dim1d, pattern::Pattern};

#[derive(Debug)]
pub struct SingleColor {
    color: SingleColorParams,
}

impl<Layout> Pattern<Dim1d, Layout> for SingleColor
where
    Layout: Layout1d,
{
    type Params = SingleColorParams;
    type Color = Hsv;

    /// Chooses the color to display
    fn new(color: SingleColorParams) -> Self {
        Self { color }
    }

    /// Generates colors for a 1D layout.
    ///
    /// This is a static, single color, so tick is only useful the first time it's called.
    /// And it will always have the same output
    fn tick(&self, _time_in_ms: u64) -> impl Iterator<Item = Self::Color> {
        let Self { color } = self;

        Layout::points().map(move |_x| Self::Color::from(color))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SingleColorParams {
    Amber,
    SoftAmber,
    CandleWarm,
    Pumpkin,
    Green,
    UltraSoftNightYellow,
    UltraSoftNightOrange,
    WarmWhite,
    WledWarmWhite,
}

impl From<&SingleColorParams> for Hsv {
    fn from(color: &SingleColorParams) -> Self {
        match color {
            SingleColorParams::Amber => Hsv::new(0.0828, 0.5922, 1.0),
            SingleColorParams::SoftAmber => Hsv::new(0.0734, 0.6941, 1.0),
            SingleColorParams::CandleWarm => Hsv::new(0.0711, 0.8, 1.0),
            SingleColorParams::Pumpkin => Hsv::new(0.0556, 0.75, 0.8),
            SingleColorParams::Green => Hsv::new(0.3333, 1.0, 1.0),
            SingleColorParams::UltraSoftNightYellow => Hsv::new(0.037, 1.0, 0.3529),
            SingleColorParams::UltraSoftNightOrange => Hsv::new(0.0, 0.8, 0.0784),
            SingleColorParams::WarmWhite => Hsv::new(0.10, 0.16, 0.12),
            SingleColorParams::WledWarmWhite => Hsv::new(0.083, 0.25, 0.12),
        }
    }
}
