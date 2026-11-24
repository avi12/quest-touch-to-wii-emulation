use std::time::Duration;

use crate::math::matrix::multiply_by_transpose;
use crate::math::vector::{norm, scale};
use crate::math::{Mat3, RadiansPerSecond};

const MIN_SINE: f64 = 1e-12;
const HALF: f64 = 0.5;

/// World-frame angular rate taking rotation `previous` to `current` over `elapsed`.
#[must_use]
pub fn rotation_rate(current: Mat3, previous: Mat3, elapsed: Duration) -> RadiansPerSecond {
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
    let is_no_rotation = sine <= MIN_SINE;
    if is_no_rotation {
        return RadiansPerSecond([0.0; 3]);
    }
    RadiansPerSecond(scale(
        sine_times_axis,
        sine.min(1.0).asin() / sine / elapsed.as_secs_f64(),
    ))
}

/// `yaw(about y) * pitch(about x) * roll(about z)`, for golden-value tests shared with the TS.
#[cfg(test)]
pub(crate) fn rotation_from_angles(yaw: f64, pitch: f64, roll: f64) -> Mat3 {
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_roll, cos_roll) = roll.sin_cos();
    let yaw_matrix = [
        [cos_yaw, 0.0, sin_yaw],
        [0.0, 1.0, 0.0],
        [-sin_yaw, 0.0, cos_yaw],
    ];
    // Transposed factors, so `multiply_by_transpose` multiplies by the factors themselves.
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
    multiply_by_transpose(
        multiply_by_transpose(yaw_matrix, pitch_transposed),
        roll_transposed,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::math::{Mat3, rotation_rate};

    const TOLERANCE: f64 = 1e-12;
    const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    const FRAME: Duration = Duration::from_millis(4);
    const YAW_ANGLE: f64 = 0.1;
    const HALF_SECOND: Duration = Duration::from_millis(500);
    const EXPECTED_YAW_RATE: f64 = 0.2;

    #[test]
    fn no_rotation_has_no_rate() {
        assert_eq!(rotation_rate(IDENTITY, IDENTITY, FRAME).0, [0.0; 3]);
    }

    #[test]
    fn yaw_about_up_axis() {
        let (sine, cosine) = YAW_ANGLE.sin_cos();
        let yawed = [[cosine, 0.0, sine], [0.0, 1.0, 0.0], [-sine, 0.0, cosine]];
        let [pitch_rate, yaw_rate, roll_rate] = rotation_rate(yawed, IDENTITY, HALF_SECOND).0;
        assert!(pitch_rate.abs() < TOLERANCE && roll_rate.abs() < TOLERANCE);
        assert!((yaw_rate - EXPECTED_YAW_RATE).abs() < TOLERANCE);
    }
}
