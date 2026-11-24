//! Dominant direction of a cloud of rotation vectors: the twist axis.

use bridge::math::{Mat3, Vec3, dot, multiply_matrix, norm, scale};

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
    if length == 0.0 {
        return vector;
    }
    scale(vector, 1.0 / length)
}

/// Dominant eigenvector and eigenvalue of a symmetric positive semi-definite 3x3 matrix, by power
/// iteration.
pub fn dominant_eigen(matrix: Mat3) -> Eigen {
    let mut unit = normalized(multiply_matrix(matrix, [1.0; 3]));
    for _step in 0..POWER_ITERATIONS {
        unit = normalized(multiply_matrix(matrix, unit));
    }
    Eigen {
        unit,
        eigenvalue: dot(multiply_matrix(matrix, unit), unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_dominant_axis() {
        let matrix = [[1.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 0.0, 2.0]];
        let eigen = dominant_eigen(matrix);
        assert!((eigen.eigenvalue - 5.0).abs() < 1e-9);
        assert!((eigen.unit[1].abs() - 1.0).abs() < 1e-9);
    }
}
