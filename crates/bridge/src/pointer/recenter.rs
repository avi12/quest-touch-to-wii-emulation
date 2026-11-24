use super::screen::is_aim_usable;
use crate::math::{Vec3, angle_between, degrees_to_radians};

const EDGE_HOLD_MS: f64 = 800.0; // pinned and still this long: the screen is where you aim now
const MS_PER_SECOND: f64 = 1000.0;
const STILL_RATE: f64 = degrees_to_radians(20.0); // rad/s below which the controller counts as held still

/// One frame's input to the edge-hold auto-recenter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EdgeSample {
    pub(crate) direction: Vec3,
    pub(crate) delta_seconds: f64,
    pub(crate) now_seconds: f64,
    pub(crate) is_pinned: bool,
}

/// Recenters when the pointer stays pinned at a screen edge while the controller is held still.
#[derive(Clone, Debug, Default)]
pub(crate) struct EdgeHold {
    previous_direction: Option<Vec3>,
    pinned_still_since: Option<f64>,
}

impl EdgeHold {
    /// Pinned at an edge while held still means the screen is not where you aim; a shake alone
    /// must not move it.
    pub(crate) fn is_pinned_and_still(&mut self, sample: &EdgeSample) -> bool {
        let direction = sample.direction;
        let turned_by = self
            .previous_direction
            .replace(direction)
            .map_or(f64::INFINITY, |previous| angle_between(previous, direction));
        let is_still = turned_by / sample.delta_seconds < STILL_RATE;
        if !sample.is_pinned || !is_still {
            self.pinned_still_since = None;
            return false;
        }
        let since = *self.pinned_still_since.get_or_insert(sample.now_seconds);
        let is_held_long_enough = (sample.now_seconds - since) * MS_PER_SECOND > EDGE_HOLD_MS;
        if !is_held_long_enough || !is_aim_usable(direction) {
            return false;
        }
        self.pinned_still_since = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AHEAD: Vec3 = [0.0, 0.0, -1.0];
    const FRAME_SECONDS: f64 = 0.004;

    fn sample(now_seconds: f64, is_pinned: bool) -> EdgeSample {
        EdgeSample {
            direction: AHEAD,
            delta_seconds: FRAME_SECONDS,
            now_seconds,
            is_pinned,
        }
    }

    #[test]
    fn fires_once_after_the_hold() {
        let mut hold = EdgeHold::default();
        assert!(!hold.is_pinned_and_still(&sample(0.0, true))); // first frame: no turn rate yet
        assert!(!hold.is_pinned_and_still(&sample(0.1, true)));
        assert!(!hold.is_pinned_and_still(&sample(0.9, true)));
        assert!(hold.is_pinned_and_still(&sample(0.95, true)));
        assert!(!hold.is_pinned_and_still(&sample(0.96, true)));
    }

    #[test]
    fn unpinning_restarts_the_hold() {
        let mut hold = EdgeHold::default();
        hold.is_pinned_and_still(&sample(0.0, true));
        hold.is_pinned_and_still(&sample(0.1, true));
        hold.is_pinned_and_still(&sample(0.5, false));
        assert!(!hold.is_pinned_and_still(&sample(1.0, true)));
    }
}
