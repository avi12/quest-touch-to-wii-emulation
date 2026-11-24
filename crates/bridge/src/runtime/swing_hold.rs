//! Swings: the pointer holds still through them, and Dolphin's motion tracking is recentred after.

use std::time::{Duration, Instant};

use crate::math::{Vec3, norm};
use crate::pointer::{PointerPosition, PointerUpdate};

/// Accel magnitude (g) that counts as a swing in progress.
const SWING_ACCEL_G: f64 = 1.5;
/// The pointer stays where it was during a swing and this long after it.
const POINTER_HOLD: Duration = Duration::from_millis(200);
/// Calm this long after a swing, then Dolphin's motion tracking is recentred.
const SWING_SETTLE: Duration = Duration::from_millis(250);

/// Remembers the last swing and the pointer from before it.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SwingHold {
    /// Last moment a swing was in progress; `None` once Dolphin was recentred after it.
    last_swing_at: Option<Instant>,
    pointer_frozen_until: Option<Instant>,
    steady_pointer: PointerPosition,
}

impl SwingHold {
    /// A hard swing leaves Dolphin's motion tracking skewed. Feed it every frame's accel (g);
    /// true once the controller has settled after a swing, when Dolphin should be recentred.
    pub(super) fn observe(&mut self, accel: Vec3, now: Instant) -> bool {
        let is_swinging = norm(accel) > SWING_ACCEL_G;
        if is_swinging {
            self.last_swing_at = Some(now);
            self.pointer_frozen_until = Some(now + POINTER_HOLD);
            return false;
        }
        let is_settled = self
            .last_swing_at
            .is_some_and(|swung_at| now.duration_since(swung_at) >= SWING_SETTLE);
        if !is_settled {
            return false;
        }
        self.last_swing_at = None;
        true
    }

    /// A swing sweeps the aim far past the screen, so the pointer keeps its pre-swing position
    /// until the swing is over.
    pub(super) fn hold(&mut self, pointer: &PointerUpdate, now: Instant) -> PointerPosition {
        let is_frozen =
            pointer.is_aimed_away || self.pointer_frozen_until.is_some_and(|until| now < until);
        if !is_frozen {
            self.steady_pointer = pointer.position;
        }
        self.steady_pointer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STILL: Vec3 = [0.0, -1.0, 0.0];
    const SWING: Vec3 = [1.2, -1.0, 0.3];

    fn aimed(x: f64, is_aimed_away: bool) -> PointerUpdate {
        PointerUpdate {
            position: PointerPosition { x, y: 0.0 },
            did_recenter: false,
            is_aimed_away,
            facing: 1.0,
            direction: [0.0, 0.0, -1.0],
        }
    }

    fn after(start: Instant, milliseconds: u64) -> Instant {
        start + Duration::from_millis(milliseconds)
    }

    #[test]
    fn recentres_once_after_the_swing_settles() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert!(!swing.observe(STILL, start));
        assert!(!swing.observe(SWING, start));
        assert!(!swing.observe(STILL, after(start, 249)));
        assert!(swing.observe(STILL, after(start, 250)));
        assert!(!swing.observe(STILL, after(start, 400)));
    }

    #[test]
    fn a_new_swing_restarts_the_settling() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        swing.observe(SWING, start);
        swing.observe(SWING, after(start, 200));
        assert!(!swing.observe(STILL, after(start, 300)));
        assert!(swing.observe(STILL, after(start, 450)));
    }

    #[test]
    fn pointer_holds_through_the_swing_and_200_ms_after() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert_eq!(swing.hold(&aimed(0.1, false), start).x, 0.1);
        swing.observe(SWING, start);
        assert_eq!(swing.hold(&aimed(0.9, false), start).x, 0.1);
        assert_eq!(swing.hold(&aimed(0.8, false), after(start, 199)).x, 0.1);
        assert_eq!(swing.hold(&aimed(0.5, false), after(start, 200)).x, 0.5);
    }

    #[test]
    fn pointer_holds_while_aimed_away() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert_eq!(swing.hold(&aimed(0.3, true), start).x, 0.0);
        assert_eq!(swing.hold(&aimed(0.3, false), start).x, 0.3);
    }
}
