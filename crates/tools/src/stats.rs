//! Summary statistics shared by the tools.

use std::fmt;

use crate::format::to_fixed;

const SPREAD_DECIMALS: usize = 2;
const PERCENT: f64 = 100.0;

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

/// `part` as a percentage of `whole` (0 when `whole` is 0).
pub fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    count_f64(part) / count_f64(whole) * PERCENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreads_a_series() {
        assert_eq!(spread_text(&[1.0, 3.0]), "1.00..3.00 sd 1.00");
        assert_eq!(spread_text(&[]), "-");
        assert!((percent(1, 4) - 25.0).abs() < f64::EPSILON);
        assert!(percent(1, 0).abs() < f64::EPSILON);
    }
}
