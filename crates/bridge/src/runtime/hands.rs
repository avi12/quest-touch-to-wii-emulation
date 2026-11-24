//! Which controller input counts this frame: the Ctrl+Alt hotkeys, the left controller's on/off
//! switch, and the pointer-test and parked overrides.

use openvr_sys::DeviceIndex;

use crate::config::PointerSource;
use crate::input::{Controllers, Controls};
use crate::motion::{LEVEL_STILL_SENSORS, Sensors};
use crate::pointer::{PointerPosition, sweep_pointer};
use crate::runtime::Bridge;
use crate::runtime::clock::since_epoch;
use crate::session::beep_switch;

/// Where a switched-off second Wii Remote points.
const PARKED_POINTER: PointerPosition = PointerPosition { x: -1.0, y: 0.0 };

/// What the pad carries for the pointer and the motion sensors.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Output {
    pub(crate) pointer: PointerPosition,
    pub(crate) sensors: Sensors,
}

impl Bridge {
    /// The second bridge with its (left) controller switched off: it reads nothing and parks.
    fn is_parked(&self) -> bool {
        self.config.is_player2() && !self.is_left_enabled
    }

    pub(crate) fn apply_hotkeys(&mut self) {
        let events = self.hotkeys.poll(win32_sys::is_key_down);
        if events.is_left_toggled {
            self.is_left_enabled = !self.is_left_enabled;
            let state = if self.is_left_enabled { "ON" } else { "OFF" };
            println!("LEFT CONTROLLER {state}");
            if !self.config.is_player2() {
                beep_switch(self.is_left_enabled); // one bridge beeps, not both
            }
        }
        if let Some(step) = events.candidate_step {
            let candidate = self.pointer.cycle_candidate(step);
            println!(
                "CANDIDATE {} {}",
                self.pointer.candidate_index(),
                candidate.name
            );
        }
    }

    pub(crate) fn read_controllers(
        &self,
        right_index: DeviceIndex,
        left_index: Option<DeviceIndex>,
    ) -> Controllers {
        let read = |index: DeviceIndex| {
            self.open_vr
                .controller_state(index)
                .map(|state| Controls::from(&state))
                .unwrap_or_default()
        };
        let right = if self.is_parked() {
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

    pub(crate) fn apply_overrides(&self, output: Output) -> Output {
        if self.is_parked() {
            // A switched-off left controller must not punch through a Nunchuk's motion.
            return Output {
                pointer: PARKED_POINTER,
                sensors: LEVEL_STILL_SENSORS,
            };
        }
        match self.config.pointer_source {
            PointerSource::Controller => output,
            PointerSource::Sweep => Output {
                pointer: sweep_pointer(since_epoch()),
                sensors: LEVEL_STILL_SENSORS,
            },
        }
    }
}
