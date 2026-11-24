//! Status lines at most every few seconds.

use std::time::{Duration, Instant};

const REPORT_INTERVAL: Duration = Duration::from_millis(3000);

/// Lets one report through per [`REPORT_INTERVAL`]; the first one at once.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Throttle {
    last_at: Option<Instant>,
}

impl Throttle {
    /// True when a report is due; it then counts as made.
    pub(crate) fn is_due(&mut self, now: Instant) -> bool {
        let is_due = self
            .last_at
            .is_none_or(|last_at| now.duration_since(last_at) > REPORT_INTERVAL);
        if is_due {
            self.last_at = Some(now);
        }
        is_due
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::runtime::throttle::{REPORT_INTERVAL, Throttle};

    #[test]
    fn reports_first_then_every_interval() {
        let start = Instant::now();
        let mut throttle = Throttle::default();
        assert!(throttle.is_due(start));
        assert!(!throttle.is_due(start + REPORT_INTERVAL));
        assert!(throttle.is_due(start + REPORT_INTERVAL + Duration::from_millis(1)));
    }
}
