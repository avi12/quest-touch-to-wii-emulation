//! DSU protocol constants, message types and request parsing.

use std::time::Duration;

use crate::wire::frame::{Magic, SourceId, event_of};

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

/// Wire id of [`Event::Version`].
const VERSION_CODE: u32 = 0x10_0000;
/// Wire id of [`Event::Ports`].
const PORTS_CODE: u32 = 0x10_0001;
/// Wire id of [`Event::Pad`].
const PAD_CODE: u32 = 0x10_0002;

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
            Self::Version => VERSION_CODE,
            Self::Ports => PORTS_CODE,
            Self::Pad => PAD_CODE,
        }
    }

    /// The event for a wire id, if known.
    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            VERSION_CODE => Some(Self::Version),
            PORTS_CODE => Some(Self::Ports),
            PAD_CODE => Some(Self::Pad),
            _ => None,
        }
    }
}

impl SourceId {
    /// The id of the server bound to `port`, unique per bridge instance.
    pub(crate) fn server(port: u16) -> Self {
        Self(SERVER_ID_BASE.wrapping_add(u32::from(port)))
    }
}

/// The event a client request asks for: it needs a full header, the `DSUC` magic and a known id.
pub(crate) fn parse_request(data: &[u8]) -> Option<Event> {
    let Some(Magic::Client) = Magic::of_packet(data) else {
        return None;
    };
    event_of(data)
}

#[cfg(test)]
mod tests {
    use crate::wire::frame::{EVENT_RANGE, HEADER_BYTES, Magic, SourceId, frame_dsu};
    use crate::wire::protocol::{Event, parse_request};

    const CLIENT: SourceId = SourceId(42);
    const REQUEST_BODY: [u8; 4] = [0; 4];
    const UNKNOWN_EVENT: u8 = 9;

    #[test]
    fn parses_known_client_requests() {
        for event in [Event::Version, Event::Ports, Event::Pad] {
            let packet = frame_dsu(Magic::Client, CLIENT, event, &REQUEST_BODY).unwrap();
            assert_eq!(parse_request(&packet), Some(event));
        }
    }

    #[test]
    fn rejects_short_foreign_or_unknown_packets() {
        let pad = frame_dsu(Magic::Client, CLIENT, Event::Pad, &[]).unwrap();
        assert_eq!(parse_request(&pad[..HEADER_BYTES - 1]), None);
        let server = frame_dsu(Magic::Server, CLIENT, Event::Pad, &[]).unwrap();
        assert_eq!(parse_request(&server), None);
        let mut unknown = pad;
        unknown[EVENT_RANGE.start] = UNKNOWN_EVENT;
        assert_eq!(parse_request(&unknown), None);
    }
}
