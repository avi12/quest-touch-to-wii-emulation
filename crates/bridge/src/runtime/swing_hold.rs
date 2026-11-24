//! Swings: the pointer follows them but is not flung to the screen edge, and Dolphin's motion
//! tracking is held recentred through them until its tilt has eased back.

use std::time::{Duration, Instant};

use crate::math::{Gravities, norm};
use crate::pointer::{PointerPosition, PointerUpdate};

/// Accel magnitude (g) that counts as a swing in progress.
const SWING_ACCEL_G: f64 = 1.5;
/// The pointer stays where it was during a swing and this long after it.
const POINTER_HOLD: Duration = Duration::from_millis(200);
/// How long after a swing Dolphin's motion pointer is still held recentred. Dolphin eases its tilt
/// back toward the accelerometer by 2% per 200 Hz update (`Accelerometer Influence`), so after
/// 0.7 s plus the 0.3 s Recenter hold under 2% of the swing's tip is left.
const IMU_SETTLE: Duration = Duration::from_millis(700);

/// Remembers the last swing and the pointer from before it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SwingHold {
    /// Last moment a swing was in progress.
    last_swing_at: Option<Instant>,
    pointer_frozen_until: Option<Instant>,
    steady_pointer: PointerPosition,
}

impl SwingHold {
    /// Whether Dolphin's motion tracking should be held recentred now. Feed it every frame's
    /// accel.
    ///
    /// Dolphin tilts its motion-pointer orientation toward the accelerometer's direction every
    /// frame (`EmulateIMUCursor`'s complementary filter, `Dynamics.cpp`). A fast sideways swing
    /// points that direction far from gravity, which tips the cursor up or down, and the tip eases
    /// back only slowly after the swing. Holding Recenter through the swing and the easing keeps
    /// the cursor where the pointer puts it; letting go earlier leaves it drifting off centre.
    pub(crate) fn observe(&mut self, accel: Gravities, now: Instant) -> bool {
        let is_swinging = norm(accel.0) > SWING_ACCEL_G;
        if is_swinging {
            self.last_swing_at = Some(now);
            self.pointer_frozen_until = Some(now + POINTER_HOLD);
            return true;
        }
        self.last_swing_at
            .is_some_and(|swung_at| now.duration_since(swung_at) < IMU_SETTLE)
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
    /// Milliseconds after the swing, around the 700 ms easing and the 200 ms pointer hold.
    const STILL_EASING_MS: u64 = 699;
    const EASED_MS: u64 = 700;
    const SECOND_SWING_MS: u64 = 200;
    const AFTER_SECOND_EASING_MS: u64 = 850;
    const SECOND_EASED_MS: u64 = 900;
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
    fn holds_recentre_through_the_swing_and_the_easing() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        assert!(!swing.observe(STILL, start));
        assert!(swing.observe(SWING, start));
        assert!(swing.observe(STILL, after(start, STILL_EASING_MS)));
        assert!(!swing.observe(STILL, after(start, EASED_MS)));
    }

    #[test]
    fn a_new_swing_restarts_the_easing() {
        let start = Instant::now();
        let mut swing = SwingHold::default();
        swing.observe(SWING, start);
        swing.observe(SWING, after(start, SECOND_SWING_MS));
        assert!(swing.observe(STILL, after(start, AFTER_SECOND_EASING_MS)));
        assert!(!swing.observe(STILL, after(start, SECOND_EASED_MS)));
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
