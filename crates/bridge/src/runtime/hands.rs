//! Which controller input counts this frame: the Ctrl+Alt hotkeys, the left controller's on/off
//! switch, and the pointer-test and parked overrides.

use super::Bridge;
use super::clock::since_epoch;
use crate::input::{Controllers, Controls};
use crate::motion::{LEVEL_STILL_SENSORS, Sensors};
use crate::pointer::{PointerPosition, sweep_pointer};
use crate::session::beep_switch;

/// Where a switched-off second Wii Remote points.
const PARKED_POINTER: PointerPosition = PointerPosition { x: -1.0, y: 0.0 };

/// What the pad carries for the pointer and the motion sensors.
#[derive(Clone, Copy, Debug)]
pub(super) struct Output {
    pub(super) pointer: PointerPosition,
    pub(super) sensors: Sensors,
}

impl Bridge {
    pub(super) fn apply_hotkeys(&mut self) {
        let events = self.hotkeys.poll(win32_sys::is_key_down);
        if events.is_left_toggled {
            self.is_left_enabled = !self.is_left_enabled;
            let state = if self.is_left_enabled { "ON" } else { "OFF" };
            println!("LEFT CONTROLLER {state}");
            if !self.config.is_player2 {
                beep_switch(self.is_left_enabled); // one bridge beeps, not both
            }
        }
        if let Some(step) = events.candidate_step.offset() {
            let candidate = self.pointer.cycle_candidate(step);
            println!(
                "CANDIDATE {} {}",
                self.pointer.candidate_index(),
                candidate.name
            );
        }
    }

    pub(super) fn read_controllers(
        &self,
        right_index: u32,
        left_index: Option<u32>,
    ) -> Controllers {
        let read = |index: u32| {
            self.open_vr
                .controller_state(index)
                .map(|state| Controls::from(&state))
                .unwrap_or_default()
        };
        let is_right_ignored = self.config.is_player2 && !self.is_left_enabled;
        let right = if is_right_ignored {
            Controls::default()
        } else {
            read(right_index)
        };
        let left = left_index
            .filter(|_index| self.is_left_enabled)
            .map(read)
            .unwrap_or_default();
        Controllers { right, left }
    }

    pub(super) fn apply_overrides(&self, output: Output) -> Output {
        if self.config.is_player2 && !self.is_left_enabled {
            // A switched-off left controller must not punch through a Nunchuk's motion.
            return Output {
                pointer: PARKED_POINTER,
                sensors: LEVEL_STILL_SENSORS,
            };
        }
        if self.config.is_pointer_test {
            return Output {
                pointer: sweep_pointer(since_epoch()),
                sensors: LEVEL_STILL_SENSORS,
            };
        }
        output
    }
}
