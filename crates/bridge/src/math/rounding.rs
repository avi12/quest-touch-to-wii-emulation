const HUNDREDTHS: f64 = 100.0;
const HALF: f64 = 0.5;

/// `Math.round(value * 100) / 100`: JavaScript rounds halves up (towards +infinity), not away from
/// zero, so -1.005 and 1.005 do not mirror; logs and telemetry keep the TypeScript bridge's values.
#[must_use]
pub fn round_hundredths(value: f64) -> f64 {
    ((value * HUNDREDTHS) + HALF).floor() / HUNDREDTHS
}

#[cfg(test)]
mod tests {
    use crate::math::round_hundredths;

    #[test]
    fn rounds_halves_up_like_javascript() {
        assert_eq!(round_hundredths(0.125), 0.13);
        assert_eq!(round_hundredths(-0.335), -0.33);
        assert_eq!(round_hundredths(-1234.5 / 100.0), -12.34);
        assert_eq!(round_hundredths(2.0), 2.0);
    }
}
