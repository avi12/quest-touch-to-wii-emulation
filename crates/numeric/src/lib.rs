//! Numbers: checked float-to-integer conversion, and JavaScript-compatible formatting.
//!
//! Rust has no `TryFrom<f64>` for integers, and the workspace denies `as`; this crate is the one
//! place that casts, after checking the value is finite and in range. Narrower integers follow with
//! the standard `TryFrom<i64>`.

pub mod format;

/// The smallest `f64` that is out of `i64` range (2^63); `i64::MIN` (−2^63) itself is exact.
const I64_LIMIT: f64 = 9_223_372_036_854_775_808.0;

/// `value` with its fraction dropped, as an `i64`; `None` when it is NaN, infinite or out of range.
pub fn truncate_to_i64(value: f64) -> Option<i64> {
    let truncated = value.trunc();
    let is_in_range = (-I64_LIMIT..I64_LIMIT).contains(&truncated);
    if !is_in_range {
        return None;
    }
    Some(truncated as i64)
}

/// `value` truncated toward zero into any integer type that `i64` converts into; `None` when it
/// does not fit.
pub fn truncate_to<Target: TryFrom<i64>>(value: f64) -> Option<Target> {
    truncate_to_i64(value).and_then(|whole| Target::try_from(whole).ok())
}

#[cfg(test)]
mod tests {
    use crate::{truncate_to, truncate_to_i64};

    #[test]
    fn truncates_toward_zero() {
        assert_eq!(truncate_to_i64(2.9), Some(2));
        assert_eq!(truncate_to_i64(-2.9), Some(-2));
    }

    #[test]
    fn rejects_values_that_do_not_fit() {
        assert_eq!(truncate_to_i64(f64::NAN), None);
        assert_eq!(truncate_to_i64(f64::INFINITY), None);
        assert_eq!(truncate_to::<u8>(256.0), None);
        assert_eq!(truncate_to::<u8>(-1.0), None);
        assert_eq!(truncate_to::<u8>(255.5), Some(255));
    }
}
