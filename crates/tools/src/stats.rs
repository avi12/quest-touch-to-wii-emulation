//! Summary statistics shared by the tools.

use std::fmt;

use numeric::format::to_fixed;

const SPREAD_DECIMALS: usize = 2;
const PERCENT_PER_WHOLE: f64 = 100.0;

/// Range and standard deviation of a series.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spread {
    pub min: f64,
    pub max: f64,
    pub deviation: f64,
}

impl Spread {
    /// The spread of `values`, or `None` when there are none.
    pub fn from_values(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let count = count_f64(values.len());
        let mean = values.iter().sum::<f64>() / count;
        let squares: f64 = values
            .iter()
            .map(|value| (value - mean) * (value - mean))
            .sum();
        Some(Self {
            min: values.iter().copied().fold(f64::INFINITY, f64::min),
            max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            deviation: (squares / count).sqrt(),
        })
    }
}

impl fmt::Display for Spread {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}..{} sd {}",
            to_fixed(self.min, SPREAD_DECIMALS),
            to_fixed(self.max, SPREAD_DECIMALS),
            to_fixed(self.deviation, SPREAD_DECIMALS),
        )
    }
}

/// `min..max sd deviation` of `values`, or `-` when there are none.
pub fn spread_text(values: &[f64]) -> String {
    Spread::from_values(values).map_or_else(|| "-".to_owned(), |spread| spread.to_string())
}

/// A count as a float; counts here are far below 2^32.
pub fn count_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// A share of a whole, 0-100.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Percent(f64);

impl Percent {
    /// `part` as a percentage of `whole` (0 when `whole` is 0).
    pub fn share(part: usize, whole: usize) -> Self {
        let is_empty = whole == 0;
        if is_empty {
            return Self(0.0);
        }
        Self(count_f64(part) / count_f64(whole) * PERCENT_PER_WHOLE)
    }

    /// The percentage as a number, 0-100.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Rounded to a whole percent, the way `toFixed(0)` prints it.
    pub fn whole_text(self) -> String {
        to_fixed(self.0, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::{Percent, spread_text};

    #[test]
    fn spreads_a_series() {
        assert_eq!(spread_text(&[1.0, 3.0]), "1.00..3.00 sd 1.00");
        assert_eq!(spread_text(&[]), "-");
    }

    #[test]
    fn takes_a_percentage() {
        assert!((Percent::share(1, 4).get() - 25.0).abs() < f64::EPSILON);
        assert!(Percent::share(1, 0).get().abs() < f64::EPSILON);
        assert_eq!(Percent::share(2, 3).whole_text(), "67");
    }
}
