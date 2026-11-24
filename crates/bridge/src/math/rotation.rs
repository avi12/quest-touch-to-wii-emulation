use std::f64::consts::PI;

use super::matrix::multiply_by_transpose;
use super::vector::{norm, scale};
use super::{Mat3, Vec3};

const HALF_TURN_DEGREES: f64 = 180.0;
const MIN_SINE: f64 = 1e-12;
const HALF: f64 = 0.5;

#[must_use]
pub const fn degrees_to_radians(degrees: f64) -> f64 {
    (degrees * PI) / HALF_TURN_DEGREES
}

#[must_use]
pub const fn radians_to_degrees(radians: f64) -> f64 {
    (radians * HALF_TURN_DEGREES) / PI
}

/// World-frame angular rate (rad/s) taking rotation `previous` to `current` over `seconds`.
#[must_use]
pub fn rotation_rate(current: Mat3, previous: Mat3, seconds: f64) -> Vec3 {
    let delta = multiply_by_transpose(current, previous);
    let sine_times_axis = scale(
        [
            delta[2][1] - delta[1][2],
            delta[0][2] - delta[2][0],
            delta[1][0] - delta[0][1],
        ],
        HALF,
    );
    let sine = norm(sine_times_axis);
    if sine <= MIN_SINE {
        return [0.0; 3];
    }
    scale(sine_times_axis, sine.min(1.0).asin() / sine / seconds)
}

/// `yaw(about y) * pitch(about x) * roll(about z)`, for golden-value tests shared with the TS.
#[cfg(test)]
pub(crate) fn rotation_from_angles(yaw: f64, pitch: f64, roll: f64) -> Mat3 {
    use super::matrix::multiply_by_transpose as times_transpose;
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_roll, cos_roll) = roll.sin_cos();
    let yaw_matrix = [
        [cos_yaw, 0.0, sin_yaw],
        [0.0, 1.0, 0.0],
        [-sin_yaw, 0.0, cos_yaw],
    ];
    // Transposed factors, so `times_transpose` multiplies by the factors themselves.
    let pitch_transposed = [
        [1.0, 0.0, 0.0],
        [0.0, cos_pitch, sin_pitch],
        [0.0, -sin_pitch, cos_pitch],
    ];
    let roll_transposed = [
        [cos_roll, sin_roll, 0.0],
        [-sin_roll, cos_roll, 0.0],
        [0.0, 0.0, 1.0],
    ];
    times_transpose(
        times_transpose(yaw_matrix, pitch_transposed),
        roll_transposed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f64 = 1e-12;
    const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    #[test]
    fn converts_angles() {
        assert!((degrees_to_radians(180.0) - PI).abs() < TOLERANCE);
        assert!((radians_to_degrees(PI / 2.0) - 90.0).abs() < TOLERANCE);
    }

    #[test]
    fn no_rotation_has_no_rate() {
        assert_eq!(rotation_rate(IDENTITY, IDENTITY, 0.004), [0.0; 3]);
    }

    #[test]
    fn yaw_about_up_axis() {
        let angle: f64 = 0.1;
        let (sine, cosine) = angle.sin_cos();
        let yawed = [[cosine, 0.0, sine], [0.0, 1.0, 0.0], [-sine, 0.0, cosine]];
        let rate = rotation_rate(yawed, IDENTITY, 0.5);
        assert!(rate[0].abs() < TOLERANCE && rate[2].abs() < TOLERANCE);
        assert!((rate[1] - 0.2).abs() < TOLERANCE);
    }
}
