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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_zero() {
        assert_eq!((0, 0, 0, 0), split(0));
    }

    #[test]
    fn splits_one() {
        assert_eq!((0, 0, 0, 1), split(1));
    }

    #[test]
    fn splits_single_digits() {
        assert_eq!((0, 0, 0, 6), split(6));
        assert_eq!((0, 0, 0, 7), split(7));
        assert_eq!((0, 0, 0, 9), split(9));
    }

    #[test]
    fn splits_ten() {
        assert_eq!((0, 0, 1, 0), split(10));
    }

    #[test]
    fn splits_seventeen() {
        assert_eq!((0, 0, 1, 7), split(17));
    }

    #[test]
    fn splits_twenty_five() {
        assert_eq!((0, 0, 2, 5), split(25));
    }

    #[test]
    fn splits_ninety_nine() {
        assert_eq!((0, 0, 9, 9), split(99));
    }

    #[test]
    fn splits_one_hundred() {
        assert_eq!((0, 1, 0, 0), split(100));
    }

    #[test]
    fn splits_two_hundred() {
        assert_eq!((0, 2, 0, 0), split(200));
    }

    #[test]
    fn splits_hundreds() {
        assert_eq!((0, 2, 3, 4), split(234));
        assert_eq!((0, 4, 8, 1), split(481));
        assert_eq!((0, 5, 2, 7), split(527));
        assert_eq!((0, 9, 9, 9), split(999));
    }

    #[test]
    fn splits_one_thousand() {
        assert_eq!((1, 0, 0, 0), split(1000));
    }

    #[test]
    fn splits_five_thousands() {
        assert_eq!((5, 0, 0, 0), split(5000));
    }

    #[test]
    fn splits_thousands() {
        assert_eq!((1, 1, 1, 1), split(1111));
        assert_eq!((1, 2, 3, 4), split(1234));
        assert_eq!((4, 4, 8, 1), split(4481));
        assert_eq!((5, 6, 7, 8), split(5678));
        assert_eq!((6, 5, 2, 7), split(6527));
        assert_eq!((9, 9, 9, 9), split(9999));
    }

    #[test]
    fn truncates_ten_thousands() {
        assert_eq!((9, 9, 9, 9), split(10_000));
    }

    #[test]
    fn truncates_even_bigger_numbers() {
        assert_eq!((9, 9, 9, 9), split(12_345));
        assert_eq!((9, 9, 9, 9), split(13_648));
        assert_eq!((9, 9, 9, 9), split(54_321));
    }
}
