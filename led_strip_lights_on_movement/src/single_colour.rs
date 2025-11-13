use blinksy::{color::Hsv, layout::Layout1d, markers::Dim1d, pattern::Pattern};

#[derive(Debug)]
pub struct SingleColour {
    colour: SingleColourParams,
}

impl<Layout> Pattern<Dim1d, Layout> for SingleColour
where
    Layout: Layout1d,
{
    type Params = SingleColourParams;
    type Color = Hsv;

    /// Creates a new SingleColourParams pattern with the specified parameters.
    fn new(colour: SingleColourParams) -> Self {
        Self { colour }
    }

    /// Generates colors for a 1D layout.
    ///
    /// The SingleColourParams pattern creates a smooth transition of hues across the layout,
    /// which shifts over time to create a flowing effect.
    fn tick(&self, _time_in_ms: u64) -> impl Iterator<Item = Self::Color> {
        let Self { colour } = self;

        Layout::points().map(move |_x| Self::Color::from(colour))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SingleColourParams {
    Amber,
    SoftAmber,
    CandleWarm,
    Pumpkin,
    Green,
    UltraSoftNightYellow,
    UltraSoftNightOrange,
    WarmWhite,
}

impl From<&SingleColourParams> for Hsv {
    fn from(colour: &SingleColourParams) -> Self {
        match colour {
            SingleColourParams::Amber => Hsv::new(0.0828, 0.5922, 1.0),
            SingleColourParams::SoftAmber => Hsv::new(0.0734, 0.6941, 1.0),
            SingleColourParams::CandleWarm => Hsv::new(0.0711, 0.8, 1.0),
            SingleColourParams::Pumpkin => Hsv::new(0.0556, 0.75, 0.8),
            SingleColourParams::Green => Hsv::new(0.3333, 1.0, 1.0),
            SingleColourParams::UltraSoftNightYellow => Hsv::new(0.037, 1.0, 0.3529),
            SingleColourParams::UltraSoftNightOrange => Hsv::new(0.0, 0.8, 0.0784),
            SingleColourParams::WarmWhite => Hsv::new(0.10, 0.16, 0.12),
        }
    }
}
