use crate::math::vector::{add, dot, scale};
use crate::math::{Mat3, Vec3};

/// `matrix * vector`.
#[must_use]
pub fn multiply_matrix(matrix: Mat3, vector: Vec3) -> Vec3 {
    matrix.map(|row| dot(row, vector))
}

/// `transpose(matrix) * vector`.
#[must_use]
pub fn multiply_transposed(matrix: Mat3, vector: Vec3) -> Vec3 {
    add(
        add(scale(matrix[0], vector[0]), scale(matrix[1], vector[1])),
        scale(matrix[2], vector[2]),
    )
}

/// `matrix * transpose(other)`.
pub(crate) fn multiply_by_transpose(matrix: Mat3, other: Mat3) -> Mat3 {
    matrix.map(|row| other.map(|other_row| dot(row, other_row)))
}

#[cfg(test)]
mod tests {
    use crate::math::Mat3;
    use crate::math::matrix::{multiply_by_transpose, multiply_matrix, multiply_transposed};

    const SAMPLE: Mat3 = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];

    #[test]
    fn multiplies_by_vector_and_transpose() {
        assert_eq!(multiply_matrix(SAMPLE, [1.0, 0.0, -1.0]), [-2.0; 3]);
        assert_eq!(multiply_transposed(SAMPLE, [1.0, 0.0, -1.0]), [-6.0; 3]);
    }

    #[test]
    fn product_with_own_transpose_is_symmetric() {
        let product = multiply_by_transpose(SAMPLE, SAMPLE);
        assert_eq!(product[0], [14.0, 32.0, 50.0]);
        assert_eq!(product[1][0], product[0][1]);
    }
}
