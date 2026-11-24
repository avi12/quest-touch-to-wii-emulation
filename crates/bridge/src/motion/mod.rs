//! Controller pose -> Wii Remote accelerometer and gyro readings in Dolphin's DSU convention.

mod sensors;
mod swing;
mod tracker;

pub use sensors::to_sensors;
pub use tracker::MotionTracker;

use crate::math::{Mat3, Vec3};

/// One `SteamVR` pose of a controller, in the world frame, independent of the binding that read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseSample {
    /// Metres.
    pub position: Vec3,
    /// Device-to-world rotation, row-major.
    pub rotation: Mat3,
    /// m/s, world frame, as `SteamVR` reports it.
    pub velocity: Vec3,
    /// rad/s, world frame, as `SteamVR` reports it.
    pub angular_velocity: Vec3,
    /// Monotonic clock reading when the pose was sampled.
    pub timestamp_seconds: f64,
}

/// Smoothed world-frame motion of a controller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    /// m/s^2.
    pub acceleration: Vec3,
    /// rad/s.
    pub angular_rate: Vec3,
}

/// Wii Remote readings in Dolphin's DSU convention: accel in g, gyro in deg/s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sensors {
    pub accel: Vec3,
    pub gyro: Vec3,
}

/// A Wii Remote lying level and still.
pub const LEVEL_STILL_SENSORS: Sensors = Sensors {
    accel: [0.0, -1.0, 0.0],
    gyro: [0.0, 0.0, 0.0],
};

#[cfg(test)]
mod tests;
