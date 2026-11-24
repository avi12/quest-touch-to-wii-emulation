//! What a device is and which hand holds it: `ETrackedDeviceClass`, `ETrackedControllerRole`.

use crate::device::code::c_enum;

c_enum! {
    /// `ETrackedDeviceClass`: what kind of device sits in a slot.
    DeviceClass {
        /// No device in the slot.
        Invalid = 0,
        /// The headset.
        Hmd = 1,
        /// A hand controller.
        Controller = 2,
        /// A generic tracker.
        GenericTracker = 3,
        /// A base station or camera.
        TrackingReference = 4,
        /// An accessory that redirects the display.
        DisplayRedirect = 5,
    }
}

c_enum! {
    /// `ETrackedControllerRole`: which hand, if any, a controller is in.
    ControllerRole {
        /// No role (not a hand controller, or not yet assigned).
        Invalid = 0,
        /// The left hand.
        LeftHand = 1,
        /// The right hand.
        RightHand = 2,
        /// A controller that opted out of hand roles.
        OptOut = 3,
        /// A treadmill.
        Treadmill = 4,
        /// A stylus.
        Stylus = 5,
    }
}

/// Which hand a controller is held in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    /// `TrackedControllerRole_LeftHand`.
    Left,
    /// `TrackedControllerRole_RightHand`.
    Right,
}

impl Hand {
    /// The controller role of this hand.
    pub const fn role(self) -> ControllerRole {
        match self {
            Self::Left => ControllerRole::LeftHand,
            Self::Right => ControllerRole::RightHand,
        }
    }
}

impl ControllerRole {
    /// The hand of a hand role, `None` for every other role.
    pub const fn hand(self) -> Option<Hand> {
        match self {
            Self::LeftHand => Some(Hand::Left),
            Self::RightHand => Some(Hand::Right),
            Self::Invalid | Self::OptOut | Self::Treadmill | Self::Stylus => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::device::kind::{ControllerRole, DeviceClass, Hand};

    #[test]
    fn codes_round_trip() {
        for class in [
            DeviceClass::Invalid,
            DeviceClass::Controller,
            DeviceClass::DisplayRedirect,
        ] {
            assert_eq!(DeviceClass::from_code(class.code()), Some(class));
        }
        assert_eq!(DeviceClass::from_code(-1), None);
        assert_eq!(
            ControllerRole::from_code(2),
            Some(ControllerRole::RightHand)
        );
    }

    #[test]
    fn hands_and_roles_agree() {
        for hand in [Hand::Left, Hand::Right] {
            assert_eq!(hand.role().hand(), Some(hand));
        }
        assert_eq!(ControllerRole::Stylus.hand(), None);
    }
}
