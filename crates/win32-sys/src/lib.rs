//! Safe wrappers for the Win32 calls the bridge needs: key state, beep, timer resolution

use std::thread;

use windows_sys::Win32::Media::timeBeginPeriod;
use windows_sys::Win32::System::Diagnostics::Debug::Beep;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

/// High bit of `GetAsyncKeyState`: the key is down right now.
const KEY_DOWN_MASK: u16 = 0x8000;

/// A Windows virtual-key code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualKey(pub i32);

/// The virtual keys the bridge's hotkeys use.
#[derive(Clone, Copy, Debug)]
pub struct Keys {
    /// Either Ctrl key.
    pub ctrl: VirtualKey,
    /// Either Alt key.
    pub alt: VirtualKey,
    /// The L key.
    pub key_l: VirtualKey,
    /// The `[` key.
    pub left_bracket: VirtualKey,
    /// The `]` key.
    pub right_bracket: VirtualKey,
}

/// Windows virtual-key codes used by the hotkeys.
pub const KEY: Keys = Keys {
    ctrl: VirtualKey(0x11),
    alt: VirtualKey(0x12),
    key_l: VirtualKey(0x4c),
    left_bracket: VirtualKey(0xdb),
    right_bracket: VirtualKey(0xdd),
};

/// True while the key is held, anywhere on the system.
pub fn is_key_down(key: VirtualKey) -> bool {
    // SAFETY: GetAsyncKeyState takes a plain integer and has no memory preconditions.
    let state = unsafe { GetAsyncKeyState(key.0) };
    state.cast_unsigned() & KEY_DOWN_MASK != 0
}

/// Non-blocking beep: `Beep` blocks for its duration, so it runs on its own thread.
pub fn beep(frequency_hz: u32, duration_ms: u32) {
    thread::spawn(move || {
        // SAFETY: Beep takes two plain integers and has no memory preconditions.
        unsafe { Beep(frequency_hz, duration_ms) };
    });
}

/// Ask Windows for finer timer ticks so a 4 ms loop really runs at 250 Hz.
/// Returns the `timeBeginPeriod` result: 0 on success.
pub fn set_timer_resolution(milliseconds: u32) -> u32 {
    // SAFETY: timeBeginPeriod takes a plain integer and has no memory preconditions.
    unsafe { timeBeginPeriod(milliseconds) }
}
