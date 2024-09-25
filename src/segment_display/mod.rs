//! module segment_display
//! Supports Miuzei 4 digits 7-segments display

use std::{thread::sleep, time::Duration};

use esp_idf_svc::{
    hal::gpio::{Output, Pin, PinDriver},
    sys::EspError,
};

pub mod numbers;

// This is the value required by the component
const DELAY_MICROSECONDS: u64 = 54;

pub struct SegmentDisplay<
    'a,
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
> {
    digit1: Driver<'a, Digit1>,
    digit2: Driver<'a, Digit2>,
    digit3: Driver<'a, Digit3>,
    digit4: Driver<'a, Digit4>,
    seg_a: Driver<'a, SegA>,
    seg_b: Driver<'a, SegB>,
    seg_c: Driver<'a, SegC>,
    seg_d: Driver<'a, SegD>,
    seg_e: Driver<'a, SegE>,
    seg_f: Driver<'a, SegF>,
    seg_g: Driver<'a, SegG>,
}

enum Digit {
    First,
    Second,
    Third,
    Fourth,
}

type Driver<'a, P> = PinDriver<'a, P, Output>;

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
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        digit1: Driver<'a, Digit1>,
        digit2: Driver<'a, Digit2>,
        digit3: Driver<'a, Digit3>,
        digit4: Driver<'a, Digit4>,
        seg_a: Driver<'a, SegA>,
        seg_b: Driver<'a, SegB>,
        seg_c: Driver<'a, SegC>,
        seg_d: Driver<'a, SegD>,
        seg_e: Driver<'a, SegE>,
        seg_f: Driver<'a, SegF>,
        seg_g: Driver<'a, SegG>,
    ) -> Result<Self, EspError> {
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
        segment_display.clear()?;
        Ok(segment_display)
    }

    pub fn clear(&mut self) -> Result<(), EspError> {
        self.seg_a.set_low()?;
        self.seg_b.set_low()?;
        self.seg_c.set_low()?;
        self.seg_d.set_low()?;
        self.seg_e.set_low()?;
        self.seg_f.set_low()?;
        self.seg_g.set_low()?;
        Ok(())
    }

    pub fn display(&mut self, number: u16) -> Result<(), EspError> {
        let (first_digit, second_digit, third_digit, fourth_digit) = split(number);

        self.display_at(Digit::First, first_digit)?;
        self.display_at(Digit::Second, second_digit)?;
        self.display_at(Digit::Third, third_digit)?;
        self.display_at(Digit::Fourth, fourth_digit)?;
        Ok(())
    }

    fn display_at(&mut self, at: Digit, number: u8) -> Result<(), EspError> {
        self.digit1.set_low()?;
        self.digit2.set_low()?;
        self.digit3.set_low()?;
        self.digit4.set_low()?;

        match at {
            Digit::First => self.digit1.set_high()?,
            Digit::Second => self.digit2.set_high()?,
            Digit::Third => self.digit3.set_high()?,
            Digit::Fourth => self.digit4.set_high()?,
        }

        self.display_digit(number)?;
        sleep(Duration::from_micros(DELAY_MICROSECONDS));

        Ok(())
    }
}

pub fn split(number: u16) -> (u8, u8, u8, u8) {
    let number = if number > 9999 { 9999 } else { number };

    (
        (number / 1000).try_into().unwrap(),
        ((number / 100) % 10).try_into().unwrap(),
        ((number / 10) % 10).try_into().unwrap(),
        (number % 10).try_into().unwrap(),
    )
}
