//! Telemetry timestamps are milliseconds since the Unix epoch, as `f64`.

use std::time::Duration;

/// Milliseconds in a second.
pub const MS_PER_SECOND: f64 = 1000.0;

/// `duration` in telemetry milliseconds.
pub fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * MS_PER_SECOND
}

/// Seconds from the timestamp `from_ms` to `to_ms`.
pub fn elapsed_seconds(from_ms: f64, to_ms: f64) -> f64 {
    (to_ms - from_ms) / MS_PER_SECOND
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::telemetry::{elapsed_seconds, millis};

    #[test]
    fn converts_between_units() {
        assert!((millis(Duration::from_millis(200)) - 200.0).abs() < f64::EPSILON);
        assert!((elapsed_seconds(1000.0, 3500.0) - 2.5).abs() < f64::EPSILON);
    }
}
