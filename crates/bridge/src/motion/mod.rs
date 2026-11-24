//! Controller pose -> Wii Remote accelerometer and gyro readings in Dolphin's DSU convention.

mod sensors;
mod swing;
mod tracker;

pub use sensors::to_sensors;
pub use tracker::MotionTracker;

use std::time::Duration;

use crate::math::{
    DegreesPerSecond, Gravities, Mat3, MetresPerSecond, MetresPerSecondSquared, RadiansPerSecond,
    Vec3,
};

/// One `SteamVR` pose of a controller, in the world frame, independent of the binding that read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseSample {
    /// Metres.
    pub position: Vec3,
    /// Device-to-world rotation, row-major.
    pub rotation: Mat3,
    /// World frame, as `SteamVR` reports it.
    pub velocity: MetresPerSecond,
    /// World frame, as `SteamVR` reports it.
    pub angular_velocity: RadiansPerSecond,
    /// Monotonic clock reading when the pose was sampled.
    pub timestamp: Duration,
}

/// Smoothed world-frame motion of a controller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    pub acceleration: MetresPerSecondSquared,
    pub angular_rate: RadiansPerSecond,
}

/// Wii Remote readings in Dolphin's DSU convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sensors {
    pub accel: Gravities,
    pub gyro: DegreesPerSecond,
}

/// A Wii Remote lying level and still.
pub const LEVEL_STILL_SENSORS: Sensors = Sensors {
    accel: Gravities([0.0, -1.0, 0.0]),
    gyro: DegreesPerSecond([0.0, 0.0, 0.0]),
};

#[cfg(test)]
mod tests;
