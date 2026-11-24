//! The axis a controller twists around ("like a screwdriver"): how the user holds it.

mod eigen;
mod recorder;
mod rotation;

pub use eigen::{Eigen, dominant_eigen};
pub use recorder::{TwistAxis, TwistRecorder};
pub use rotation::relative_rotation;
