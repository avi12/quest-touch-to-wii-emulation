//! Global Ctrl+Alt hotkeys (never clash with Dolphin's), reported once per key press.

use win32_sys::{KEY, VirtualKey};

/// Which way Ctrl+Alt+] / [ cycles the pointing axis this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateStep {
    Previous,
    Stay,
    Next,
}

impl CandidateStep {
    /// The step for `Pointer::cycle_candidate`, or `None` when staying.
    pub fn offset(self) -> Option<isize> {
        match self {
            Self::Previous => Some(-1),
            Self::Stay => None,
            Self::Next => Some(1),
        }
    }
}

/// What the hotkeys asked for this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeyEvents {
    /// Ctrl+Alt+L: switch the left controller on or off.
    pub is_left_toggled: bool,
    pub candidate_step: CandidateStep,
}

#[derive(Clone, Copy, Debug, Default)]
struct RisingEdge {
    was_down: bool,
}

impl RisingEdge {
    fn update(&mut self, is_down: bool) -> bool {
        let is_edge = is_down && !self.was_down;
        self.was_down = is_down;
        is_edge
    }
}

/// Polls the Ctrl+Alt hotkeys; the pointing-axis keys only work in the first bridge.
#[derive(Clone, Copy, Debug)]
pub struct Hotkeys {
    is_player2: bool,
    toggle: RisingEdge,
    next: RisingEdge,
    previous: RisingEdge,
}

impl Hotkeys {
    pub fn new(is_player2: bool) -> Self {
        Self {
            is_player2,
            toggle: RisingEdge::default(),
            next: RisingEdge::default(),
            previous: RisingEdge::default(),
        }
    }

    /// `is_down` reads one key (`win32_sys::is_key_down`, or a test double).
    pub fn poll(&mut self, is_down: impl Fn(VirtualKey) -> bool) -> HotkeyEvents {
        let is_modifier_held = is_down(KEY.ctrl) && is_down(KEY.alt);
        let is_cycle_enabled = is_modifier_held && !self.is_player2;
        let is_next_pressed = self
            .next
            .update(is_cycle_enabled && is_down(KEY.right_bracket));
        let is_previous_pressed = self
            .previous
            .update(is_cycle_enabled && is_down(KEY.left_bracket));
        let candidate_step = if is_next_pressed {
            CandidateStep::Next
        } else if is_previous_pressed {
            CandidateStep::Previous
        } else {
            CandidateStep::Stay
        };
        HotkeyEvents {
            is_left_toggled: self.toggle.update(is_modifier_held && is_down(KEY.key_l)),
            candidate_step,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(keys: &[VirtualKey]) -> impl Fn(VirtualKey) -> bool {
        move |key| keys.contains(&key)
    }

    #[test]
    fn reports_each_press_once() {
        let mut hotkeys = Hotkeys::new(false);
        let chord = [KEY.ctrl, KEY.alt, KEY.key_l];
        assert!(hotkeys.poll(held(&chord)).is_left_toggled);
        assert!(!hotkeys.poll(held(&chord)).is_left_toggled);
        assert!(!hotkeys.poll(held(&[])).is_left_toggled);
        assert!(hotkeys.poll(held(&chord)).is_left_toggled);
    }

    #[test]
    fn needs_both_modifiers() {
        let mut hotkeys = Hotkeys::new(false);
        assert!(!hotkeys.poll(held(&[KEY.ctrl, KEY.key_l])).is_left_toggled);
    }

    #[test]
    fn cycles_only_in_the_first_bridge() {
        let next = [KEY.ctrl, KEY.alt, KEY.right_bracket];
        let previous = [KEY.ctrl, KEY.alt, KEY.left_bracket];
        assert_eq!(
            Hotkeys::new(false).poll(held(&next)).candidate_step,
            CandidateStep::Next
        );
        assert_eq!(
            Hotkeys::new(false).poll(held(&previous)).candidate_step,
            CandidateStep::Previous
        );
        let mut second = Hotkeys::new(true);
        assert_eq!(second.poll(held(&next)).candidate_step, CandidateStep::Stay);
        assert!(
            second
                .poll(held(&[KEY.ctrl, KEY.alt, KEY.key_l]))
                .is_left_toggled
        );
    }
}
