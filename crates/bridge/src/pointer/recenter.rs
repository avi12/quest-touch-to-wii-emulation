use std::time::Duration;

use crate::math::{Degrees, Radians, Vec3, angle_between};
use crate::pointer::screen::is_aim_usable;

const EDGE_HOLD: Duration = Duration::from_millis(800); // pinned and still this long: the screen is where you aim now
const STILL_RATE: Radians = Degrees(20.0).to_radians(); // per second; slower counts as held still

/// One frame's input to the edge-hold auto-recenter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EdgeSample {
    pub(crate) direction: Vec3,
    pub(crate) delta: Duration,
    /// Monotonic clock.
    pub(crate) now: Duration,
    pub(crate) is_pinned: bool,
}

/// Recenters when the pointer stays pinned at a screen edge while the controller is held still.
#[derive(Clone, Debug, Default)]
pub(crate) struct EdgeHold {
    previous_direction: Option<Vec3>,
    pinned_still_since: Option<Duration>,
}

impl EdgeHold {
    /// Pinned at an edge while held still means the screen is not where you aim; a shake alone
    /// must not move it.
    pub(crate) fn is_pinned_and_still(&mut self, sample: &EdgeSample) -> bool {
        let direction = sample.direction;
        let turned_by = self
            .previous_direction
            .replace(direction)
            .map_or(f64::INFINITY, |previous| {
                angle_between(previous, direction).0
            });
        let is_still = turned_by / sample.delta.as_secs_f64() < STILL_RATE.0;
        let is_pinned_still = sample.is_pinned && is_still;
        if !is_pinned_still {
            self.pinned_still_since = None;
            return false;
        }
        let since = *self.pinned_still_since.get_or_insert(sample.now);
        let is_held_long_enough = sample.now.saturating_sub(since) > EDGE_HOLD;
        let is_due = is_held_long_enough && is_aim_usable(direction);
        if !is_due {
            return false;
        }
        self.pinned_still_since = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::math::Vec3;
    use crate::pointer::recenter::{EdgeHold, EdgeSample};

    const AHEAD: Vec3 = [0.0, 0.0, -1.0];
    const FRAME: Duration = Duration::from_millis(4);
    const START: Duration = Duration::ZERO;
    const HOLD_STARTS: Duration = Duration::from_millis(100);
    /// Exactly the hold after it started: not yet longer than it.
    const HOLD_ENDS: Duration = Duration::from_millis(900);
    const PAST_HOLD: Duration = Duration::from_millis(950);
    const AFTER_FIRING: Duration = Duration::from_millis(960);
    const UNPINNED_AT: Duration = Duration::from_millis(500);
    const REPINNED_AT: Duration = Duration::from_secs(1);

    fn sample(now: Duration, is_pinned: bool) -> EdgeSample {
        EdgeSample {
            direction: AHEAD,
            delta: FRAME,
            now,
            is_pinned,
        }
    }

    #[test]
    fn fires_once_after_the_hold() {
        let mut hold = EdgeHold::default();
        assert!(!hold.is_pinned_and_still(&sample(START, true))); // first frame: no turn rate yet
        assert!(!hold.is_pinned_and_still(&sample(HOLD_STARTS, true)));
        assert!(!hold.is_pinned_and_still(&sample(HOLD_ENDS, true)));
        assert!(hold.is_pinned_and_still(&sample(PAST_HOLD, true)));
        assert!(!hold.is_pinned_and_still(&sample(AFTER_FIRING, true)));
    }

    #[test]
    fn unpinning_restarts_the_hold() {
        let mut hold = EdgeHold::default();
        hold.is_pinned_and_still(&sample(START, true));
        hold.is_pinned_and_still(&sample(HOLD_STARTS, true));
        hold.is_pinned_and_still(&sample(UNPINNED_AT, false));
        assert!(!hold.is_pinned_and_still(&sample(REPINNED_AT, true)));
    }
}
