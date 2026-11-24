//! Swings: the pointer follows them but is not flung to the screen edge, and Dolphin's motion tracking is held recentred
//! through them and recentred again once they settle.

use std::time::{Duration, Instant};

use crate::math::{Gravities, norm};
use crate::pointer::{PointerPosition, PointerUpdate};

/// Accel magnitude (g) that counts as a swing in progress.
const SWING_ACCEL_G: f64 = 1.5;
/// The pointer stays where it was during a swing and this long after it.
const POINTER_HOLD: Duration = Duration::from_millis(200);
/// Calm this long after a swing, then Dolphin's motion tracking is recentred.
const SWING_SETTLE: Duration = Duration::from_millis(250);

/// Remembers the last swing and the pointer from before it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SwingHold {
    /// Last moment a swing was in progress; `None` once Dolphin was recentred after it.
    last_swing_at: Option<Instant>,
    pointer_frozen_until: Option<Instant>,
    steady_pointer: PointerPosition,
}

impl SwingHold {
    /// Whether Dolphin's motion tracking should be recentred now. Feed it every frame's accel.
    ///
    /// Dolphin tilts its motion-pointer orientation toward the accelerometer's direction every
    /// frame (`EmulateIMUCursor`'s complementary filter, `Dynamics.cpp`). A fast sideways swing
    /// points that direction far from gravity, which tips the cursor up or down; holding Recenter
    /// through the swing cancels the tip as it builds. True on every swing frame, and once more
    /// when the controller has settled after it.
    pub(crate) fn observe(&mut self, accel: Gravities, now: Instant) -> bool {
        let is_swinging = norm(accel.0) > SWING_ACCEL_G;
        if is_swinging {
            self.last_swing_at = Some(now);
            self.pointer_frozen_until = Some(now + POINTER_HOLD);
            return true;
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

    /// The pointer follows a fast move while it stays on screen; when a swing sweeps it to the
    /// edge, it keeps its last on-screen position until the swing is over.
    pub(crate) fn hold(&mut self, pointer: &PointerUpdate, now: Instant) -> PointerPosition {
        let is_swinging = self.pointer_frozen_until.is_some_and(|until| now < until);
        let is_swept_off_screen = is_swinging && pointer.position.is_at_edge();
        let is_frozen = pointer.is_aimed_away || is_swept_off_screen;
        if !is_frozen {
            self.steady_pointer = pointer.position;
        }
        self.steady_pointer
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::math::Gravities;
    use crate::pointer::{PointerPosition, PointerUpdate};
    use crate::runtime::swing_hold::SwingHold;

    const STILL: Gravities = Gravities([0.0, -1.0, 0.0]);
    const SWING: Gravities = Gravities([1.2, -1.0, 0.3]);
    /// Milliseconds after the swing, around the 250 ms settle and the 200 ms pointer hold.
    const JUST_UNSETTLED_MS: u64 = 249;
    const SETTLED_MS: u64 = 250;
    const LONG_AFTER_MS: u64 = 400;
    const SECOND_SWING_MS: u64 = 200;
    const AFTER_SECOND_SWING_MS: u64 = 300;
    const SECOND_SETTLED_MS: u64 = 450;
    const JUST_HELD_MS: u64 = 199;
    const RELEASED_MS: u64 = 200;
    const STEADY_X: f64 = 0.1;
    const FOLLOWED_X: f64 = 0.9;
    const EDGE_X: f64 = 0.95;
    const RELEASED_X: f64 = 0.5;
    const AWAY_X: f64 = 0.3;

    fn aimed(x: f64, is_aimed_away: bool) -> PointerUpdate {
        PointerUpdate {
            position: PointerPosition { x, y: 0.0 },
            has_recentered: false,
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
        assert!(swing.observe(SWING, start));
        assert!(!swing.observe(STILL, after(start, JUST_UNSETTLED_MS)));
        assert!(swing.observe(STILL, after(start, SETTLED_MS)));
        assert!(!swing.observe(STILL, after(start, LONG_AFTER_MS)));
    }

    #[test]
    fn a_new_swing_restarts_the_settling() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        swing.observe(SWING, start);
        swing.observe(SWING, after(start, SECOND_SWING_MS));
        assert!(!swing.observe(STILL, after(start, AFTER_SECOND_SWING_MS)));
        assert!(swing.observe(STILL, after(start, SECOND_SETTLED_MS)));
    }

    #[test]
    fn pointer_follows_a_fast_move_but_not_off_the_screen() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert_eq!(swing.hold(&aimed(STEADY_X, false), start).x, STEADY_X);
        swing.observe(SWING, start);
        assert_eq!(swing.hold(&aimed(FOLLOWED_X, false), start).x, FOLLOWED_X);
        let held = swing.hold(&aimed(EDGE_X, false), after(start, JUST_HELD_MS));
        assert_eq!(held.x, FOLLOWED_X);
        let released = swing.hold(&aimed(EDGE_X, false), after(start, RELEASED_MS));
        assert_eq!(released.x, EDGE_X);
        assert_eq!(
            swing
                .hold(&aimed(RELEASED_X, false), after(start, RELEASED_MS))
                .x,
            RELEASED_X
        );
    }

    #[test]
    fn pointer_holds_while_aimed_away() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert_eq!(swing.hold(&aimed(AWAY_X, true), start).x, 0.0);
        assert_eq!(swing.hold(&aimed(AWAY_X, false), start).x, AWAY_X);
    }
}
