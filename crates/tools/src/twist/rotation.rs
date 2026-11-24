//! The small rotation between two controller poses, in the controller's own frame.

use bridge::math::{multiply_transposed, scale};
use geometry::{Mat3, Vec3};

const HALF: f64 = 0.5;

/// The rotation from `previous` to `current`, expressed in the controller frame, as a
/// small-angle rotation vector (the skew part of `delta = transpose(previous) * current`).
pub fn relative_rotation(previous: Mat3, current: Mat3) -> Vec3 {
    let [[c00, c01, c02], [c10, c11, c12], [c20, c21, c22]] = current;
    // Column k of delta is transpose(previous) times column k of current: delta[r][k] = col_k[r].
    let [col_0, col_1, col_2] = [[c00, c10, c20], [c01, c11, c21], [c02, c12, c22]]
        .map(|column| multiply_transposed(previous, column));
    let [_d00, d10, d20] = col_0;
    let [d01, _d11, d21] = col_1;
    let [d02, d12, _d22] = col_2;
    scale([d21 - d12, d02 - d20, d10 - d01], HALF)
}

#[cfg(test)]
mod tests {
    use crate::twist::relative_rotation;

    const ANGLE: f64 = 0.01;

    #[test]
    fn small_roll_about_z_is_a_z_vector() {
        let angle = ANGLE;
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let roll = [
            [angle.cos(), -angle.sin(), 0.0],
            [angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ];
        let [x, y, roll_part] = relative_rotation(identity, roll);
        assert!(x.abs() < 1e-12 && y.abs() < 1e-12);
        assert!((roll_part - angle.sin()).abs() < 1e-12);
    }
}
