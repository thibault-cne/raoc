use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

/// A valid part number of advent (i.e. either 1 or 2).
///
/// # Display
/// This value displays as a two digit number.
///
/// ```
/// # use advent_of_code::Part;
/// let part = Part::new(1).unwrap();
/// assert_eq!(part.to_string(), "1")
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Part(u8);

impl Part {
    /// Creates a [`Part`] from the provided value if it's in the valid range,
    /// returns [`None`] otherwise.
    pub fn new(part: u8) -> Option<Self> {
        if part != 1 && part != 2 {
            return None;
        }
        Some(Self(part))
    }

    // Not part of the public API
    #[doc(hidden)]
    pub const fn __new_unchecked(part: u8) -> Self {
        Self(part)
    }

    /// Converts the [`Part`] into an [`u8`].
    pub fn into_inner(self) -> u8 {
        self.0
    }
}

impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:01}", self.0)
    }
}

impl PartialEq<u8> for Part {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u8> for Part {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

/* -------------------------------------------------------------------------- */

impl FromStr for Part {
    type Err = PartFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let day = s.parse().map_err(|_| PartFromStrError)?;
        Self::new(day).ok_or(PartFromStrError)
    }
}

/// An error which can be returned when parsing a [`Day`].
#[derive(Debug)]
pub struct PartFromStrError;

impl Error for PartFromStrError {}

impl Display for PartFromStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expecting a part, either 1 or 2")
    }
}

/* -------------------------------------------------------------------------- */

/// Creates a [`Part`] value in a const context.
#[macro_export]
macro_rules! part {
    ($part:expr) => {{
        const _ASSERT: () = assert!(
            $part == 1 || $part == 2,
            concat!(
                "invalid day number `",
                $part,
                "`, expecting a value between 1 and 25"
            ),
        );
        $crate::Part::__new_unchecked($part)
    }};
    () => {
        const PART_ONE: advent_of_code::Part = advent_of_code::part!(1);
        const PART_TWO: advent_of_code::Part = advent_of_code::part!(2);
    };
}
