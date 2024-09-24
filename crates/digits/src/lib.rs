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
