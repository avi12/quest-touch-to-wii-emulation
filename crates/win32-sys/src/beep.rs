//! A non-blocking beep through kernel32's `Beep`.

use std::thread;
use std::time::Duration;

use crate::whole_milliseconds;

// SAFETY: the signature matches kernel32's export as `utilapiset.h` declares it (two `DWORD`s in,
// `BOOL` out).
#[link(name = "kernel32")]
unsafe extern "system" {
    fn Beep(frequency: u32, duration_ms: u32) -> i32;
}

/// A tone frequency in hertz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hertz(pub u32);

/// Plays a tone without blocking: `Beep` blocks for its duration, so it runs on its own thread.
pub fn beep(frequency: Hertz, duration: Duration) {
    let milliseconds = whole_milliseconds(duration);
    thread::spawn(move || {
        // SAFETY: Beep takes two plain integers and has no memory preconditions.
        unsafe { Beep(frequency.0, milliseconds) };
    });
}
