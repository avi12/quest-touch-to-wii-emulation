//! Vector, matrix and rotation helpers shared by the motion and pointer modules, typed units, and
//! the one-euro filter. `Vec3` and `Mat3` come from `geometry`, their one home.

mod filter;
mod matrix;
mod rotation;
mod rounding;
mod units;
mod vector;

pub use filter::{OneEuro, OneEuroOptions};
pub use geometry::{Mat3, Vec3};
pub use matrix::{multiply_matrix, multiply_transposed};
#[cfg(test)]
pub(crate) use rotation::rotation_from_angles;
pub use rotation::rotation_rate;
pub use rounding::round_hundredths;
pub use units::{
    Degrees, DegreesPerSecond, Gravities, MetresPerSecond, MetresPerSecondSquared, Radians,
    RadiansPerSecond,
};
pub use vector::{
    WORLD_UP, add, angle_between, cross, dot, norm, normalize, remove_component, scale, subtract,
};
