use blinksy::{color::Srgb, layout::Layout1d, markers::Dim1d, pattern::Pattern};

#[derive(Debug)]
pub struct SingleColour {
    colour: SingleColourParams,
}

impl<Layout> Pattern<Dim1d, Layout> for SingleColour
where
    Layout: Layout1d,
{
    type Params = SingleColourParams;
    type Color = Srgb;

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
}

impl From<&SingleColourParams> for Srgb {
    fn from(colour: &SingleColourParams) -> Self {
        match colour {
            SingleColourParams::Amber => Srgb {
                red: 255.0,
                green: 179.0,
                blue: 104.0,
            },
            SingleColourParams::SoftAmber => Srgb {
                red: 255.0,
                green: 156.0,
                blue: 78.0,
            },
            SingleColourParams::CandleWarm => Srgb {
                red: 255.0,
                green: 138.0,
                blue: 51.0,
            },
            SingleColourParams::Pumpkin => Srgb {
                red: 204.0,
                green: 102.0,
                blue: 51.0,
            },
            SingleColourParams::Green => Srgb {
                red: 0.0,
                green: 255.0,
                blue: 0.0,
            },
            SingleColourParams::UltraSoftNightYellow => Srgb {
                red: 90.0,
                green: 20.0,
                blue: 0.0,
            },
            SingleColourParams::UltraSoftNightOrange => Srgb {
                red: 20.0,
                green: 4.0,
                blue: 4.0,
            },
        }
    }
}
