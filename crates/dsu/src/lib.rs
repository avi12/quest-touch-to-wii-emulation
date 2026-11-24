//! DSU (Cemuhook) pad server as Dolphin reads it.
//!
//! Protocol: <https://github.com/v1993/cemuhook-protocol>. Dolphin-specific behaviour (read from
//! Dolphin's `DualShockUDPClient.cpp`) is noted where it matters; see `docs/how-it-works.md` §2.

mod buttons;
mod clients;
mod frame;
mod pad;
mod protocol;
mod server;

pub use buttons::{Buttons1, Buttons2};
pub use frame::{FrameError, Magic, frame_dsu};
pub use pad::{PadState, stick_byte};
pub use protocol::Event;
pub use server::Server;
