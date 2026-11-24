//! Controller state: sticks, triggers and buttons.

use crate::device::index::DeviceIndex;
use crate::ffi::table::{RawControllerState, SystemTable, len_u32};

/// `k_EButton_System`.
const SYSTEM_BIT: u32 = 0;
/// `k_EButton_ApplicationMenu`.
const APPLICATION_MENU_BIT: u32 = 1;
/// `k_EButton_A`.
const A_BIT: u32 = 7;
/// `k_EButton_Axis0` (`k_EButton_SteamVR_Touchpad`).
const AXIS0_BIT: u32 = 32;

/// A button of a Touch controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    /// A (right) or X (left): `k_EButton_A`.
    Primary,
    /// B (right) or Y (left): `k_EButton_ApplicationMenu`.
    Secondary,
    /// Stick click: `k_EButton_Axis0`.
    Click,
    /// System / menu button: `k_EButton_System`.
    System,
}

impl Button {
    /// The `EVRButtonId`: the button's bit in `ulButtonPressed`.
    fn bit(self) -> u32 {
        match self {
            Self::System => SYSTEM_BIT,
            Self::Secondary => APPLICATION_MENU_BIT,
            Self::Primary => A_BIT,
            Self::Click => AXIS0_BIT,
        }
    }
}

/// Sticks, triggers and buttons of one controller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControllerState {
    /// Stick (x, y), each -1..1.
    pub stick: [f64; 2],
    /// Index trigger, 0..1.
    pub trig: f64,
    /// Grip, 0..1.
    pub grip: f64,
    /// `ulButtonPressed`: one bit per `EVRButtonId`.
    pressed: u64,
}

impl ControllerState {
    /// Whether `button` is held.
    pub fn is_down(&self, button: Button) -> bool {
        self.pressed & (1_u64 << button.bit()) != 0
    }
}

impl From<&RawControllerState> for ControllerState {
    fn from(raw: &RawControllerState) -> Self {
        let [[stick_x, stick_y], [trig, _], [grip, _], _, _] = raw.axes;
        Self {
            stick: [f64::from(stick_x), f64::from(stick_y)],
            trig: f64::from(trig),
            grip: f64::from(grip),
            pressed: raw.button_pressed,
        }
    }
}

/// The raw state of device `index`, or `None` when `SteamVR` has none for it.
pub(crate) fn read_raw(system: &SystemTable, index: DeviceIndex) -> Option<RawControllerState> {
    let mut raw = RawControllerState::default();
    let size = len_u32(size_of::<RawControllerState>());
    // SAFETY: `raw` is a valid `VRControllerState_t` of the size passed.
    let has_state = unsafe { (system.controller_state)(index.get(), &raw mut raw, size) };
    has_state.then_some(raw)
}

#[cfg(test)]
mod tests {
    use crate::device::controller::{Button, ControllerState};
    use crate::ffi::table::RawControllerState;

    /// Stick (0.5, -0.25), trigger 0.75, grip fully closed.
    const AXES: [[f32; 2]; 5] = [
        [0.5, -0.25],
        [0.75, 0.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 0.0],
    ];

    #[test]
    fn decodes_axes_and_buttons() {
        let raw = RawControllerState {
            packet_number: 1,
            button_pressed: (1 << Button::Primary.bit()) | (1 << Button::Click.bit()),
            button_touched: 0,
            axes: AXES,
        };
        let state = ControllerState::from(&raw);
        assert_eq!(state.stick, [0.5, -0.25]);
        assert_eq!(state.trig, 0.75);
        assert_eq!(state.grip, 1.0);
        assert!(state.is_down(Button::Primary) && state.is_down(Button::Click));
        assert!(!state.is_down(Button::Secondary) && !state.is_down(Button::System));
    }
}
