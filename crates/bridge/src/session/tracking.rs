//! Beeps and logs when the second (left) controller connects or disconnects.

use std::time::{Duration, Instant};

/// A change must hold this long (ignores tracking blips).
const HOLD: Duration = Duration::from_millis(1000);
const CONNECTED_BEEP_HZ: u32 = 1200;
const DISCONNECTED_BEEP_HZ: u32 = 500;
const BEEP_MS: u32 = 200;

/// The high beep for "on / connected", the low one for "off / disconnected".
pub fn beep_switch(is_on: bool) {
    let frequency_hz = if is_on {
        CONNECTED_BEEP_HZ
    } else {
        DISCONNECTED_BEEP_HZ
    };
    win32_sys::beep(frequency_hz, BEEP_MS);
}

/// Follows whether the left controller is tracked, debounced by [`HOLD`].
#[derive(Clone, Copy, Debug)]
pub struct SecondControllerWatcher {
    is_seen: bool,
    changed_at: Instant,
}

impl SecondControllerWatcher {
    pub fn new(now: Instant) -> Self {
        Self {
            is_seen: false,
            changed_at: now,
        }
    }

    /// Beeps and logs once a change has held long enough.
    pub fn update(&mut self, is_tracked: bool, now: Instant) {
        let Some(is_connected) = self.settle(is_tracked, now) else {
            return;
        };
        beep_switch(is_connected);
        let state = if is_connected {
            "CONNECTED"
        } else {
            "DISCONNECTED"
        };
        println!("SECOND CONTROLLER {state}");
    }

    /// The new state once a change has held for longer than [`HOLD`], else `None`.
    fn settle(&mut self, is_tracked: bool, now: Instant) -> Option<bool> {
        if is_tracked == self.is_seen {
            self.changed_at = now;
            return None;
        }
        if now.duration_since(self.changed_at) <= HOLD {
            return None;
        }
        self.is_seen = is_tracked;
        Some(is_tracked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_a_change_only_after_it_holds() {
        let start = Instant::now();
        let mut watcher = SecondControllerWatcher::new(start);
        assert_eq!(watcher.settle(true, start + HOLD), None);
        assert_eq!(
            watcher.settle(true, start + HOLD + Duration::from_millis(1)),
            Some(true)
        );
        assert_eq!(watcher.settle(true, start + HOLD * 3), None);
    }

    #[test]
    fn a_blip_restarts_the_hold() {
        let start = Instant::now();
        let mut watcher = SecondControllerWatcher::new(start);
        let blip = start + Duration::from_millis(900);
        assert_eq!(watcher.settle(false, blip), None);
        assert_eq!(
            watcher.settle(true, blip + Duration::from_millis(900)),
            None
        );
    }
}
