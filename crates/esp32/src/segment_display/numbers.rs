use esp_idf_svc::{hal::gpio::Pin, sys::EspError};

use super::SegmentDisplay;

impl<'a, Digit1, Digit2, Digit3, Digit4, SegA, SegB, SegC, SegD, SegE, SegF, SegG>
    SegmentDisplay<'a, Digit1, Digit2, Digit3, Digit4, SegA, SegB, SegC, SegD, SegE, SegF, SegG>
where
    Digit1: Pin,
    Digit2: Pin,
    Digit3: Pin,
    Digit4: Pin,
    SegA: Pin,
    SegB: Pin,
    SegC: Pin,
    SegD: Pin,
    SegE: Pin,
    SegF: Pin,
    SegG: Pin,
{
    pub(super) fn display_digit(&mut self, number: u8) -> Result<(), EspError> {
        match number {
            1 => self.one(),
            2 => self.two(),
            3 => self.three(),
            4 => self.four(),
            5 => self.five(),
            6 => self.six(),
            7 => self.seven(),
            8 => self.eight(),
            9 => self.nine(),
            _ => self.zero(),
        }
    }

    fn zero(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_low()?;
        self.seg_f.set_low()?;
        self.seg_g.set_high()?;
        Ok(())
    }

    fn one(&mut self) -> Result<(), EspError> {
        self.seg_a.set_high()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_high()?;
        self.seg_e.set_high()?;
        self.seg_f.set_high()?;
        self.seg_g.set_high()?;
        Ok(())
    }

    fn two(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_high()?;
        self.seg_d.set_low()?;
        self.seg_e.set_low()?;
        self.seg_f.set_high()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn three(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_high()?;
        self.seg_f.set_high()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn four(&mut self) -> Result<(), EspError> {
        self.seg_a.set_high()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_high()?;
        self.seg_e.set_high()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn five(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_high()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_high()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn six(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_high()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_low()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn seven(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_high()?;
        self.seg_e.set_high()?;
        self.seg_f.set_high()?;
        self.seg_g.set_high()?;
        Ok(())
    }

    fn eight(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_low()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    fn nine(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_high()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }
}
