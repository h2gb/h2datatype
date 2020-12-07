use simple_error::{bail, SimpleResult};
use std::ops::Range;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Alignment {
    /// Don't align at all
    None,

    /// Each field is padded until its length is a multiple of the padding
    /// Length.. so 0..1 aligned to 4 will be 0..4, and 1..2 aligned to 4 will
    /// be 1..5
    Loose(u64),

    /// Only pad after, but error out if the start isn't aligned.
    Strict(u64),
}

impl Alignment {
    fn round_up(number: u64, multiple: u64) -> u64 {
        if multiple == 0 {
            return number;
        }

        let remainder = number % multiple;
        if remainder == 0 {
            return number;
        }

        number - remainder + multiple
    }

    // fn round_down(number: u64, multiple: u64) -> u64 {
    //     if multiple == 0 {
    //         return number;
    //     }

    //     number - (number % multiple)
    // }

    pub fn align(self, range: Range<u64>) -> SimpleResult<Range<u64>> {
        if range.end < range.start {
            bail!("Range ends before it starts");
        }

        match self {
            Self::None => Ok(range),
            Self::Loose(m) => {
                let new_size = Self::round_up(range.end - range.start, m);
                Ok(range.start..(range.start + new_size))
            },
            Self::Strict(m) => {
                if m != 0 && (range.start % m != 0) {
                    bail!("Alignment error");
                }

                let new_size = Self::round_up(range.end - range.start, m);
                Ok(range.start..(range.start + new_size))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    #[test]
    fn test_none() -> SimpleResult<()> {
        let tests: Vec<(Range<u64>, Range<u64>)> = vec![
            //       value       expected
            (         0..0,          0..0),
            (        0..10,         0..10),
            (         1..5,          1..5),
            (         2..8,          2..8),
            (10200..100000, 10200..100000),
        ];

        for (value, expected) in tests {
            assert_eq!(expected, Alignment::None.align(value)?);
        }

        Ok(())
    }

    #[test]
    fn test_loose() -> SimpleResult<()> {
        let tests: Vec<(Range<u64>, u64, Range<u64>)> = vec![
            //  value  multiple  expected
            (    0..0,        0,     0..0),
            (    0..0,        4,     0..0),
            (    0..1,        4,     0..4),
            (    0..2,        4,     0..4),
            (    1..3,        4,     1..5),
            (    3..4,        4,     3..7),
            (    5..5,        4,     5..5),
            (   1..10,      789,   1..790),
            (4..10200,    10000, 4..20004),
        ];

        for (value, multiple, expected) in tests {
            assert_eq!(expected, Alignment::Loose(multiple).align(value)?);
        }

        Ok(())
    }

    #[test]
    fn test_strict() -> SimpleResult<()> {
        let good_tests: Vec<(Range<u64>, u64, Range<u64>)> = vec![
            //      value  multiple  expected
            (    0..0,        0,         0..0),
            (    0..1,        4,         0..4),
            (    0..2,        4,         0..4),
            (    0..3,        4,         0..4),
            (    4..4,        4,         4..4),
            (    0..5,        4,         0..8),
            (    0..10,       789,       0..789),
            (    0..10200,    10000,     0..20000),
            (10000..10200,    10000, 10000..20000),
        ];

        for (value, multiple, expected) in good_tests {
            assert_eq!(expected, Alignment::Strict(multiple).align(value)?);
        }

        let bad_tests: Vec<(Range<u64>, u64)> = vec![
            //   value      multiple
            (    2..3,            4),
            (    1..1,            4),
            (    5..100,          4),
            (    4..5,            8),
            (    3..4,            4),
            (    5..5,            4),
            (    5..10,         789),
            ( 9999..10200,    10000),
            (10001..10200,    10000),
        ];

        for (value, multiple) in bad_tests {
            assert!(Alignment::Strict(multiple).align(value).is_err());
        }

        Ok(())
    }

}
