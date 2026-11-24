//! Dominant direction of a cloud of rotation vectors: the twist axis.

use bridge::math::{dot, multiply_matrix, norm, scale};
use geometry::{Mat3, Vec3};

const POWER_ITERATIONS: usize = 200;

/// The dominant eigenvector (unit length) and its eigenvalue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eigen {
    pub unit: Vec3,
    pub eigenvalue: f64,
}

/// `vector` scaled to unit length; the zero vector stays zero.
fn normalized(vector: Vec3) -> Vec3 {
    let length = norm(vector);
    let is_zero = length == 0.0;
    if is_zero {
        return vector;
    }
    scale(vector, 1.0 / length)
}

/// Dominant eigenvector and eigenvalue of a symmetric positive semi-definite 3x3 matrix, by power
/// iteration.
pub fn dominant_eigen(matrix: Mat3) -> Eigen {
    let start = normalized(multiply_matrix(matrix, [1.0; 3]));
    let unit = (0..POWER_ITERATIONS).fold(start, |unit, _iteration| {
        normalized(multiply_matrix(matrix, unit))
    });
    Eigen {
        unit,
        eigenvalue: dot(multiply_matrix(matrix, unit), unit),
    }
}

#[cfg(test)]
mod tests {
    use geometry::Mat3;

    use crate::twist::dominant_eigen;

    /// Diagonal, so its dominant axis is y with eigenvalue 5.
    const MATRIX: Mat3 = [[1.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 0.0, 2.0]];

    #[test]
    fn finds_the_dominant_axis() {
        let eigen = dominant_eigen(MATRIX);
        assert!((eigen.eigenvalue - 5.0).abs() < 1e-9);
        assert!((eigen.unit[1].abs() - 1.0).abs() < 1e-9);
    }
}
