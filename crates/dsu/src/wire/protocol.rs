//! DSU protocol constants and request parsing.

use std::time::Duration;

use crate::wire::frame::HEADER_BYTES;

/// Protocol version written in every header and in the version reply.
pub(crate) const PROTOCOL_VERSION: u16 = 1001;
/// Base of the per-instance server id (`SERVER_ID_BASE + port`).
pub(crate) const SERVER_ID_BASE: u32 = 0x7011_c0de;
/// Dolphin re-requests the pad regularly; a client is dropped after this long without a request.
pub(crate) const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);
/// How long the PS button (Dolphin's IMU "Recenter") is held after the server starts.
pub(crate) const RECENTER_HOLD: Duration = Duration::from_secs(1);
/// Slot 0, connected, model "full gyro", connection USB, zero MAC, battery full.
pub(crate) const SLOT_INFO: [u8; 11] = [0, 2, 2, 1, 0, 0, 0, 0, 0, 1, 5];

/// DSU message types (the event id after the 16-byte header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Protocol version information.
    Version,
    /// Information about connected controllers.
    Ports,
    /// Actual controller data.
    Pad,
}

impl Event {
    /// The event id on the wire.
    pub const fn code(self) -> u32 {
        match self {
            Self::Version => 0x10_0000,
            Self::Ports => 0x10_0001,
            Self::Pad => 0x10_0002,
        }
    }

    /// The event for a wire id, if known.
    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            0x10_0000 => Some(Self::Version),
            0x10_0001 => Some(Self::Ports),
            0x10_0002 => Some(Self::Pad),
            _ => None,
        }
    }
}

/// The event a client request asks for: it needs a full header, the `DSUC` magic and a known id.
pub(crate) fn parse_request(data: &[u8]) -> Option<Event> {
    let (header, _body) = data.split_first_chunk::<HEADER_BYTES>()?;
    let (magic, rest) = header.split_first_chunk::<4>()?;
    if magic != b"DSUC" {
        return None;
    }
    let (_fields, event) = rest.split_last_chunk::<4>()?;
    Event::from_code(u32::from_le_bytes(*event))
}

#[cfg(test)]
mod tests {
    use super::{Event, parse_request};
    use crate::wire::frame::{Magic, frame_dsu};

    #[test]
    fn parses_known_client_requests() {
        for event in [Event::Version, Event::Ports, Event::Pad] {
            let packet = frame_dsu(Magic::Client, 42, event, &[0; 4]).unwrap();
            assert_eq!(parse_request(&packet), Some(event));
        }
    }

    #[test]
    fn rejects_short_foreign_or_unknown_packets() {
        let pad = frame_dsu(Magic::Client, 42, Event::Pad, &[]).unwrap();
        assert_eq!(parse_request(&pad[..19]), None);
        let server = frame_dsu(Magic::Server, 42, Event::Pad, &[]).unwrap();
        assert_eq!(parse_request(&server), None);
        let mut unknown = pad;
        unknown[16] = 9;
        assert_eq!(parse_request(&unknown), None);
    }
}
