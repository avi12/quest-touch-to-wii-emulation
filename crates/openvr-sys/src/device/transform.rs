//! Decoding `SteamVR`'s 3x4 row-major matrices into a rotation and a translation.

use geometry::{Mat3, Vec3};

/// `SteamVR`'s `HmdMatrix34_t`: three rows of rotation plus a translation column.
pub(crate) type Matrix34 = [[f32; 4]; 3];

/// A component's transform relative to the controller's tracked pose: 3x3 rotation rows plus
/// origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentTransform {
    /// Rotation, row-major.
    pub rot: Mat3,
    /// Translation (the fourth column of the 3x4 matrix).
    pub origin: Vec3,
}

impl ComponentTransform {
    /// Splits a 3x4 matrix into its rotation rows and translation column.
    pub(crate) fn from_matrix(matrix: &Matrix34) -> Self {
        Self {
            rot: matrix.map(|[col0, col1, col2, _]| [col0, col1, col2].map(f64::from)),
            origin: matrix.map(|[_, _, _, translation]| f64::from(translation)),
        }
    }
}

/// Widens an `HmdVector3_t`.
pub(crate) fn to_vec3(floats: [f32; 3]) -> Vec3 {
    floats.map(f64::from)
}

#[cfg(test)]
mod tests {
    use crate::device::transform::{ComponentTransform, Matrix34, to_vec3};

    const MATRIX: Matrix34 = [
        [1.0, 2.0, 3.0, 10.0],
        [4.0, 5.0, 6.0, 20.0],
        [7.0, 8.0, 9.0, 30.0],
    ];

    #[test]
    fn splits_rotation_rows_and_translation_column() {
        let transform = ComponentTransform::from_matrix(&MATRIX);
        assert_eq!(
            transform.rot,
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
        );
        assert_eq!(transform.origin, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn widens_vectors() {
        assert_eq!(to_vec3([0.5, -1.0, 2.0]), [0.5, -1.0, 2.0]);
    }
}
