//! Vector, matrix and rotation helpers shared by the motion and pointer modules, plus the one-euro filter.

mod filter;
mod matrix;
mod rotation;
mod rounding;
mod vector;

pub use filter::{OneEuro, OneEuroOptions};
pub use matrix::{multiply_matrix, multiply_transposed};
#[cfg(test)]
pub(crate) use rotation::rotation_from_angles;
pub use rotation::{degrees_to_radians, radians_to_degrees, rotation_rate};
pub use rounding::round_hundredths;
pub use vector::{
    WORLD_UP, add, angle_between, cross, dot, norm, normalize, remove_component, scale, subtract,
};

/// A 3-vector `[x, y, z]`.
pub type Vec3 = [f64; 3];
/// A 3x3 matrix stored row-major: `matrix[row][column]`.
pub type Mat3 = [Vec3; 3];
