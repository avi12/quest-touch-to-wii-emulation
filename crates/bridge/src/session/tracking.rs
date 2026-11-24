//! Beeps and logs when the second (left) controller connects or disconnects.

use std::time::{Duration, Instant};

use win32_sys::Hertz;

/// A change must hold this long (ignores tracking blips).
const HOLD: Duration = Duration::from_millis(1000);
const CONNECTED_TONE: Hertz = Hertz(1200);
const DISCONNECTED_TONE: Hertz = Hertz(500);
const BEEP_LENGTH: Duration = Duration::from_millis(200);

/// The high beep for "on / connected", the low one for "off / disconnected".
pub fn beep_switch(is_on: bool) {
    let tone = if is_on {
        CONNECTED_TONE
    } else {
        DISCONNECTED_TONE
    };
    win32_sys::beep(tone, BEEP_LENGTH);
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
        let is_unchanged = is_tracked == self.is_seen;
        if is_unchanged {
            self.changed_at = now;
            return None;
        }
        let is_settling = now.duration_since(self.changed_at) <= HOLD;
        if is_settling {
            return None;
        }
        self.is_seen = is_tracked;
        Some(is_tracked)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::session::SecondControllerWatcher;
    use crate::session::tracking::HOLD;

    const BLIP: Duration = Duration::from_millis(900);
    const LONG_AFTER: u32 = 3;

    #[test]
    fn reports_a_change_only_after_it_holds() {
        let start = Instant::now();
        let mut watcher = SecondControllerWatcher::new(start);
        assert_eq!(watcher.settle(true, start + HOLD), None);
        assert_eq!(
            watcher.settle(true, start + HOLD + Duration::from_millis(1)),
            Some(true)
        );
        assert_eq!(watcher.settle(true, start + HOLD * LONG_AFTER), None);
    }

    #[test]
    fn a_blip_restarts_the_hold() {
        let start = Instant::now();
        let mut watcher = SecondControllerWatcher::new(start);
        let blip = start + BLIP;
        assert_eq!(watcher.settle(false, blip), None);
        assert_eq!(watcher.settle(true, blip + BLIP), None);
    }
}
