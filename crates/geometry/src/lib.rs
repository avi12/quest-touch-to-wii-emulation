//! Vectors and 3x3 rotation matrices shared by every crate of the bridge.
//!
//! The single home of these types: `SteamVR` poses, the DSU pad, the motion and pointer maths and
//! the tools all speak them, so none converts between look-alike arrays.

/// A 3D vector in metres, metres per second, radians per second or g, by context.
pub type Vec3 = [f64; 3];

/// A 3x3 rotation matrix, row-major: `matrix[row][column]`.
pub type Mat3 = [Vec3; 3];
