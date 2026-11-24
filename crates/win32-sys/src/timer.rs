//! Finer system timer ticks through winmm's `timeBeginPeriod` / `timeEndPeriod`.

use std::time::Duration;

use crate::whole_milliseconds;

// SAFETY: the signatures match winmm's exports as `timeapi.h` declares them (`UINT` in, `MMRESULT`
// out).
#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(period_ms: u32) -> u32;
    fn timeEndPeriod(period_ms: u32) -> u32;
}

/// `TIMERR_NOERROR`.
const TIMERR_NOERROR: u32 = 0;

/// A finer timer resolution, held until dropped (which restores the previous one).
#[derive(Debug)]
#[must_use = "the resolution is restored as soon as this is dropped"]
pub struct TimerResolution {
    milliseconds: u32,
}

/// Asks Windows for timer ticks of `resolution`, so a 4 ms loop really runs at 250 Hz; `None`
/// when Windows refuses the period.
pub fn set_timer_resolution(resolution: Duration) -> Option<TimerResolution> {
    let milliseconds = whole_milliseconds(resolution);
    // SAFETY: timeBeginPeriod takes a plain integer and has no memory preconditions.
    let result = unsafe { timeBeginPeriod(milliseconds) };
    (result == TIMERR_NOERROR).then_some(TimerResolution { milliseconds })
}

impl Drop for TimerResolution {
    fn drop(&mut self) {
        // SAFETY: it ends the period this value began, with the same plain integer.
        unsafe { timeEndPeriod(self.milliseconds) };
    }
}
