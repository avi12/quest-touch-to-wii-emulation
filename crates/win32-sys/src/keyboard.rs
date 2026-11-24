//! System-wide key state through user32's `GetAsyncKeyState`.

// SAFETY: the signature matches user32's export as `winuser.h` declares it (`int` in, `SHORT`
// out).
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
}

/// High bit of `GetAsyncKeyState`: the key is down right now.
const KEY_DOWN_MASK: u16 = 0x8000;

/// `VK_CONTROL`, `VK_LCONTROL`, `VK_RCONTROL`: Windows reports the generic code for either side,
/// but the sided ones are checked too so Right Ctrl always counts.
const CONTROL_CODES: [i32; 3] = [0x11, 0xa2, 0xa3];
/// `VK_MENU`, `VK_LMENU`, `VK_RMENU`: Alt on either side (Right Alt is `AltGr` on many layouts).
const ALT_CODES: [i32; 3] = [0x12, 0xa4, 0xa5];
/// The L key's virtual-key code (its ASCII letter).
const VK_L: i32 = 0x4c;
/// `VK_OEM_4`.
const VK_OEM_4: i32 = 0xdb;
/// `VK_OEM_6`.
const VK_OEM_6: i32 = 0xdd;

/// The keys the bridge's hotkeys use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// Either Ctrl key.
    Control,
    /// Either Alt key.
    Alt,
    /// The L key.
    KeyL,
    /// The `[` key (US layout).
    LeftBracket,
    /// The `]` key (US layout).
    RightBracket,
}

impl VirtualKey {
    /// The Windows virtual-key codes that count as this key.
    const fn codes(self) -> &'static [i32] {
        match self {
            Self::Control => &CONTROL_CODES,
            Self::Alt => &ALT_CODES,
            Self::KeyL => &[VK_L],
            Self::LeftBracket => &[VK_OEM_4],
            Self::RightBracket => &[VK_OEM_6],
        }
    }
}

/// True while the key (on either side, for Ctrl and Alt) is held, anywhere on the system.
pub fn is_key_down(key: VirtualKey) -> bool {
    key.codes().iter().any(|code| is_code_down(*code))
}

fn is_code_down(code: i32) -> bool {
    // SAFETY: GetAsyncKeyState takes a plain integer and has no memory preconditions.
    let state = unsafe { GetAsyncKeyState(code) };
    state.cast_unsigned() & KEY_DOWN_MASK != 0
}
