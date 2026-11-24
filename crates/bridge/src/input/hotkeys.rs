//! Global Ctrl+Alt hotkeys (never clash with Dolphin's), reported once per key press.

use openvr_sys::Hand;
use win32_sys::VirtualKey;

use crate::pointer::CandidateStep;

/// What the hotkeys asked for this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeyEvents {
    /// Ctrl+Alt+L: switch the left controller on or off.
    pub is_left_toggled: bool,
    /// Ctrl+Alt+] / [: cycle the pointing axis; `None` when neither was pressed.
    pub candidate_step: Option<CandidateStep>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RisingEdge {
    is_held: bool,
}

impl RisingEdge {
    fn update(&mut self, is_down: bool) -> bool {
        let is_edge = is_down && !self.is_held;
        self.is_held = is_down;
        is_edge
    }
}

/// Polls the Ctrl+Alt hotkeys; the pointing-axis keys only work in the right-hand bridge (Wii
/// Remote 1).
#[derive(Clone, Copy, Debug)]
pub struct Hotkeys {
    hand: Hand,
    toggle: RisingEdge,
    next: RisingEdge,
    previous: RisingEdge,
}

impl Hotkeys {
    /// Hotkeys for the bridge that reads the `hand` controller.
    pub fn new(hand: Hand) -> Self {
        Self {
            hand,
            toggle: RisingEdge::default(),
            next: RisingEdge::default(),
            previous: RisingEdge::default(),
        }
    }

    /// `is_down` reads one key (`win32_sys::is_key_down`, or a test double).
    pub fn poll(&mut self, is_down: impl Fn(VirtualKey) -> bool) -> HotkeyEvents {
        let is_modifier_held = is_down(VirtualKey::Control) && is_down(VirtualKey::Alt);
        let is_cycle_enabled = is_modifier_held && self.hand == Hand::Right;
        let is_next_pressed = self
            .next
            .update(is_cycle_enabled && is_down(VirtualKey::RightBracket));
        let is_previous_pressed = self
            .previous
            .update(is_cycle_enabled && is_down(VirtualKey::LeftBracket));
        let candidate_step = if is_next_pressed {
            Some(CandidateStep::Next)
        } else if is_previous_pressed {
            Some(CandidateStep::Previous)
        } else {
            None
        };
        HotkeyEvents {
            is_left_toggled: self
                .toggle
                .update(is_modifier_held && is_down(VirtualKey::KeyL)),
            candidate_step,
        }
    }
}

#[cfg(test)]
mod tests {
    use openvr_sys::Hand;
    use win32_sys::VirtualKey::{self, Alt, Control, KeyL, LeftBracket, RightBracket};

    use crate::input::Hotkeys;
    use crate::pointer::CandidateStep;

    fn held(keys: &[VirtualKey]) -> impl Fn(VirtualKey) -> bool {
        move |key| keys.contains(&key)
    }

    #[test]
    fn reports_each_press_once() {
        let mut hotkeys = Hotkeys::new(Hand::Right);
        let chord = [Control, Alt, KeyL];
        assert!(hotkeys.poll(held(&chord)).is_left_toggled);
        assert!(!hotkeys.poll(held(&chord)).is_left_toggled);
        assert!(!hotkeys.poll(held(&[])).is_left_toggled);
        assert!(hotkeys.poll(held(&chord)).is_left_toggled);
    }

    #[test]
    fn needs_both_modifiers() {
        let mut hotkeys = Hotkeys::new(Hand::Right);
        assert!(!hotkeys.poll(held(&[Control, KeyL])).is_left_toggled);
    }

    #[test]
    fn cycles_only_in_the_right_hand_bridge() {
        let next = [Control, Alt, RightBracket];
        let previous = [Control, Alt, LeftBracket];
        assert_eq!(
            Hotkeys::new(Hand::Right).poll(held(&next)).candidate_step,
            Some(CandidateStep::Next)
        );
        assert_eq!(
            Hotkeys::new(Hand::Right)
                .poll(held(&previous))
                .candidate_step,
            Some(CandidateStep::Previous)
        );
        let mut second = Hotkeys::new(Hand::Left);
        assert_eq!(second.poll(held(&next)).candidate_step, None);
        assert!(second.poll(held(&[Control, Alt, KeyL])).is_left_toggled);
    }
}
