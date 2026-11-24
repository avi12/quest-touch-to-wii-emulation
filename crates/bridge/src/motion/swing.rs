use crate::math::{Vec3, norm, scale, subtract};

const SWING_GAIN: f64 = 1.3; // swing acceleration passed to Dolphin (gated by SWING_LOW/HIGH so gentle aiming cannot bend the motion pointer's pitch)
const SWING_LOW: f64 = 0.6;
const SWING_HIGH: f64 = 1.2; // g: below LOW the motion is ignored (aiming), above HIGH it is passed in full
const MAX_SWING_G: f64 = 3.0; // a real Wii Remote's accelerometer saturates near 3 g; far larger values corrupt Dolphin's motion tracking

/// The part of the specific force that is not gravity, gated and saturated like a Wii Remote's.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Swing {
    /// g, Wiimote frame, each axis capped at `MAX_SWING_G`.
    pub(crate) acceleration: Vec3,
    /// 0 while aiming, 1 in a full swing.
    pub(crate) factor: f64,
}

/// Each axis of a real Wii Remote's accelerometer saturates on its own, so the cap is per axis,
/// not on the total.
fn limit_axis(value: f64) -> f64 {
    value.clamp(-MAX_SWING_G, MAX_SWING_G)
}

/// Both inputs in g, in the Wiimote frame.
pub(crate) fn measure_swing(specific_force: Vec3, gravity: Vec3) -> Swing {
    let swing = subtract(specific_force, gravity);
    let factor = ((norm(swing) - SWING_LOW) / (SWING_HIGH - SWING_LOW)).clamp(0.0, 1.0);
    let boosted = scale(swing, SWING_GAIN * factor);
    Swing {
        acceleration: boosted.map(limit_axis),
        factor,
    }
}
