//! DSU (Cemuhook) pad server as Dolphin reads it.
//!
//! Protocol: <https://github.com/v1993/cemuhook-protocol>. Dolphin-specific behaviour (read from
//! Dolphin's `DualShockUDPClient.cpp`) is noted where it matters; see `docs/how-it-works.md` §2.

mod port;
mod server;
mod wire;

pub use port::Port;
pub use server::Server;
pub use wire::analog::{Shoulders, Stick};
pub use wire::buttons::{Buttons1, Buttons2};
pub use wire::frame::{FrameError, Magic, SourceId, frame_dsu};
pub use wire::pad::PadState;
pub use wire::protocol::Event;
