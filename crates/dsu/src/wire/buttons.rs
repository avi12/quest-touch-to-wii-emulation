//! DSU digital button bytes. Dolphin only reads Share/Options/L3/R3 from these; everything else
//! comes from the analog fields, which the pad encoder mirrors from these bits.

use std::ops::{BitOr, BitOrAssign};

/// Declares a one-byte button set with its named bits.
macro_rules! button_set {
    ($(#[$doc:meta])* $name:ident { $($(#[$bit_doc:meta])* $bit:ident = $value:literal,)* }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct $name(pub u8);

        impl $name {
            /// No button held.
            pub const NONE: Self = Self(0);
            $($(#[$bit_doc])* pub const $bit: Self = Self($value);)*

            /// Whether every bit of a non-empty `other` is set in `self`.
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0 && other.0 != 0
            }
        }

        impl BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
    };
}

button_set! {
    /// First DSU button byte: Share, L3, R3, Options and the D-pad.
    Buttons1 {
        /// Share.
        SHARE = 0x01,
        /// Left stick click.
        STICK_L3 = 0x02,
        /// Right stick click.
        STICK_R3 = 0x04,
        /// Options.
        OPTIONS = 0x08,
        /// D-pad up.
        DPAD_UP = 0x10,
        /// D-pad right.
        DPAD_RIGHT = 0x20,
        /// D-pad down.
        DPAD_DOWN = 0x40,
        /// D-pad left.
        DPAD_LEFT = 0x80,
    }
}

button_set! {
    /// Second DSU button byte: shoulders, triggers and the face buttons.
    Buttons2 {
        /// L2 (left trigger).
        TRIGGER_L2 = 0x01,
        /// R2 (right trigger).
        TRIGGER_R2 = 0x02,
        /// L1 (left shoulder).
        SHOULDER_L1 = 0x04,
        /// R1 (right shoulder).
        SHOULDER_R1 = 0x08,
        /// Face X (Square).
        FACE_X = 0x10,
        /// Face A (Cross).
        FACE_A = 0x20,
        /// Face B (Circle).
        FACE_B = 0x40,
        /// Face Y (Triangle).
        FACE_Y = 0x80,
    }
}

#[cfg(test)]
mod tests {
    use crate::wire::buttons::{Buttons1, Buttons2};

    #[test]
    fn bits_combine_and_test() {
        let held = Buttons1::DPAD_UP | Buttons1::SHARE;
        assert_eq!(held.0, 0x11);
        assert!(held.contains(Buttons1::DPAD_UP));
        assert!(!held.contains(Buttons1::DPAD_LEFT));
        assert!(!held.contains(Buttons1::NONE));
        let mut face = Buttons2::FACE_A;
        face |= Buttons2::FACE_Y;
        assert_eq!(face, Buttons2(0xa0));
    }
}
