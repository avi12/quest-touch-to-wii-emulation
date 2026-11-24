//! System-wide key state through user32's `GetAsyncKeyState`.

// SAFETY: the signature matches user32's export as `winuser.h` declares it (`int` in, `SHORT`
// out).
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
}

/// High bit of `GetAsyncKeyState`: the key is down right now.
const KEY_DOWN_MASK: u16 = 0x8000;

/// `VK_CONTROL`.
const VK_CONTROL: i32 = 0x11;
/// `VK_MENU`.
const VK_MENU: i32 = 0x12;
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
    /// The Windows virtual-key code.
    const fn code(self) -> i32 {
        match self {
            Self::Control => VK_CONTROL,
            Self::Alt => VK_MENU,
            Self::KeyL => VK_L,
            Self::LeftBracket => VK_OEM_4,
            Self::RightBracket => VK_OEM_6,
        }
    }
}

/// True while the key is held, anywhere on the system.
pub fn is_key_down(key: VirtualKey) -> bool {
    // SAFETY: GetAsyncKeyState takes a plain integer and has no memory preconditions.
    let state = unsafe { GetAsyncKeyState(key.code()) };
    state.cast_unsigned() & KEY_DOWN_MASK != 0
}
