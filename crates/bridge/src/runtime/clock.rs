//! Wall-clock readings: the DSU timestamp, telemetry and the pointer sweep use the Unix epoch.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Time since the Unix epoch (zero if the clock is set before it).
pub(crate) fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

/// `Date.now()`: time since the Unix epoch in whole milliseconds.
pub(crate) fn wall_clock() -> Duration {
    Duration::from_millis(u64::try_from(since_epoch().as_millis()).unwrap_or(u64::MAX))
}
