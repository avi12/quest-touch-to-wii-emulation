//! Safe wrappers for the Win32 calls the bridge needs: key state, beep, timer resolution

mod beep;
mod keyboard;
mod timer;

pub use beep::{Hertz, beep};
pub use keyboard::{VirtualKey, is_key_down};
pub use timer::{TimerResolution, set_timer_resolution};

/// A duration as the whole milliseconds a Win32 call takes, saturating.
fn whole_milliseconds(duration: std::time::Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}
