//! Stick and shoulder bytes: the analog fields of a pad snapshot.

/// Largest stick byte (full right / full up).
const STICK_MAX: f64 = 255.0;
/// Half the byte range: maps -1..1 onto 0..255.
const STICK_HALF_RANGE: f64 = 127.5;

/// One stick as DSU bytes: 128 = centre, x 255 = right, y 255 = up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stick {
    /// Horizontal byte.
    pub x: u8,
    /// Vertical byte (255 = up, as Dolphin reads it).
    pub y: u8,
}

impl Stick {
    /// The stick at rest.
    pub const CENTRE: Self = Self { x: 128, y: 128 };

    /// A stick from axes in -1..1 (y up); NaN maps to 0 as the TypeScript bridge's buffer write
    /// did.
    pub fn from_unit(x: f64, y: f64) -> Self {
        Self {
            x: axis_byte(x),
            y: axis_byte(y),
        }
    }

    pub(crate) const fn bytes(self) -> [u8; 2] {
        [self.x, self.y]
    }
}

impl Default for Stick {
    fn default() -> Self {
        Self::CENTRE
    }
}

/// The analog shoulder and trigger bytes, 0 = released, 255 = fully pressed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Shoulders {
    /// R1 (right shoulder).
    pub right_shoulder: u8,
    /// L1 (left shoulder).
    pub left_shoulder: u8,
    /// R2 (right trigger).
    pub right_trigger: u8,
    /// L2 (left trigger).
    pub left_trigger: u8,
}

impl Shoulders {
    /// The bytes in DSU field order: R1, L1, R2, L2.
    pub(crate) const fn bytes(self) -> [u8; 4] {
        [
            self.right_shoulder,
            self.left_shoulder,
            self.right_trigger,
            self.left_trigger,
        ]
    }
}

/// One axis -1..1 -> 0..255 (128 = centre).
fn axis_byte(value: f64) -> u8 {
    let scaled = ((value + 1.0) * STICK_HALF_RANGE)
        .round()
        .clamp(0.0, STICK_MAX);
    numeric::truncate_to::<u8>(scaled).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use crate::wire::analog::{Stick, axis_byte};

    #[test]
    fn axis_bytes_match_typescript() {
        assert_eq!(
            [-2.0, -1.0, -0.5, 0.0, 0.001, 0.5, 1.0, 2.0, f64::NAN].map(axis_byte),
            [0, 0, 64, 128, 128, 191, 255, 255, 0]
        );
    }

    #[test]
    fn rest_is_centre() {
        assert_eq!(Stick::from_unit(0.0, 0.0), Stick::CENTRE);
        assert_eq!(Stick::default(), Stick::CENTRE);
    }
}
