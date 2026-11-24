//! One controller's sticks, triggers and buttons, decoupled from the `SteamVR` binding.

use openvr_sys::{Button, ControllerState};

use crate::input::deadzone::remove_dead_zone;

/// The buttons a [`Controls`] remembers, in bit order.
const BUTTONS: [Button; 4] = [
    Button::Primary,
    Button::Secondary,
    Button::Click,
    Button::System,
];

/// Sticks, triggers and buttons of one controller; `Default` is nothing held.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    /// Stick (x, y), each -1..1.
    pub stick: [f64; 2],
    /// Index trigger, 0..1.
    pub trig: f64,
    /// Grip, 0..1.
    pub grip: f64,
    /// One bit per entry of [`BUTTONS`].
    held: u8,
}

fn bit(button: Button) -> u8 {
    BUTTONS
        .iter()
        .position(|known| *known == button)
        .and_then(|index| u32::try_from(index).ok())
        .map_or(0, |index| 1 << index)
}

impl Controls {
    /// Whether `button` is held: Primary = A / X, Secondary = B / Y, Click = stick click,
    /// System = menu.
    pub fn is_down(&self, button: Button) -> bool {
        self.held & bit(button) != 0
    }

    /// The same controls with `buttons` held as well.
    #[must_use]
    pub fn holding(self, buttons: &[Button]) -> Self {
        let held = buttons
            .iter()
            .fold(self.held, |held, button| held | bit(*button));
        Self { held, ..self }
    }
}

impl From<&ControllerState> for Controls {
    fn from(state: &ControllerState) -> Self {
        let held = BUTTONS
            .into_iter()
            .filter(|button| state.is_down(*button))
            .fold(0, |held, button| held | bit(button));
        Self {
            stick: remove_dead_zone(state.stick),
            trig: state.trig,
            grip: state.grip,
            held,
        }
    }
}

/// Both controllers as one bridge sees them: `right` is the Wii Remote this bridge serves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controllers {
    pub right: Controls,
    pub left: Controls,
}
