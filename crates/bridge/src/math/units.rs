//! Units the bridge must not mix up: angles in degrees or radians, accelerations in m/s^2 or g,
//! angular rates in rad/s or deg/s.

use std::f64::consts::PI;

use crate::math::Vec3;

const HALF_TURN_DEGREES: f64 = 180.0;

/// An angle in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Degrees(pub f64);

/// An angle in radians.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Radians(pub f64);

impl Degrees {
    #[must_use]
    pub const fn to_radians(self) -> Radians {
        Radians((self.0 * PI) / HALF_TURN_DEGREES)
    }
}

impl Radians {
    #[must_use]
    pub const fn to_degrees(self) -> Degrees {
        Degrees((self.0 * HALF_TURN_DEGREES) / PI)
    }

    #[must_use]
    pub fn tan(self) -> f64 {
        self.0.tan()
    }
}

/// Declares a 3-vector tagged with its unit, so a value in one unit cannot be passed as another.
macro_rules! vector_quantity {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name(pub Vec3);
    };
}

vector_quantity!(
    /// Velocity in metres per second.
    MetresPerSecond
);
vector_quantity!(
    /// Acceleration in metres per second squared.
    MetresPerSecondSquared
);
vector_quantity!(
    /// Acceleration in standard gravities (g), as a Wii Remote's accelerometer reports it.
    Gravities
);
vector_quantity!(
    /// Angular rate in radians per second.
    RadiansPerSecond
);
vector_quantity!(
    /// Angular rate in degrees per second, as a Wii Remote's gyro reports it.
    DegreesPerSecond
);

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use crate::math::{Degrees, Radians};

    const TOLERANCE: f64 = 1e-12;
    const HALF_TURN: Degrees = Degrees(180.0);
    const QUARTER_TURN_DEGREES: f64 = 90.0;

    #[test]
    fn converts_angles() {
        assert!((HALF_TURN.to_radians().0 - PI).abs() < TOLERANCE);
        assert!((Radians(FRAC_PI_2).to_degrees().0 - QUARTER_TURN_DEGREES).abs() < TOLERANCE);
    }
}
