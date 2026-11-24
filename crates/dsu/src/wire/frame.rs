//! DSU datagram framing: 16 header bytes, CRC32 over the whole packet, event id, body.

use std::fmt;
use std::ops::Range;

use crate::wire::crc::Crc32;
use crate::wire::protocol::{Event, PROTOCOL_VERSION};

/// Header bytes including the event id.
pub(crate) const HEADER_BYTES: usize = 20;
/// Where the magic sits in the header.
const MAGIC_RANGE: Range<usize> = 0..4;
/// Where the CRC sits in the header.
const CRC_RANGE: Range<usize> = 8..12;
/// Where the event id sits in the header.
pub(crate) const EVENT_RANGE: Range<usize> = 16..20;
/// The length field counts the event id as well as the body.
const EVENT_ID_BYTES: usize = EVENT_RANGE.end - EVENT_RANGE.start;

/// Who sent a datagram, as its four magic bytes say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Magic {
    /// `DSUC`: a client request.
    Client,
    /// `DSUS`: a server response.
    Server,
}

impl Magic {
    const fn bytes(self) -> &'static [u8; 4] {
        match self {
            Self::Client => b"DSUC",
            Self::Server => b"DSUS",
        }
    }

    /// The sender a datagram's first bytes name, if they are a DSU magic.
    pub(crate) fn of_packet(packet: &[u8]) -> Option<Self> {
        let magic = packet.get(MAGIC_RANGE)?;
        [Self::Client, Self::Server]
            .into_iter()
            .find(|candidate| candidate.bytes() == magic)
    }
}

/// The sender id in every header: each server and client picks its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub u32);

/// The body does not fit the header's 16-bit length field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameError {
    /// The body length that was asked for.
    pub body_len: usize,
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "DSU body of {} bytes is too long", self.body_len)
    }
}

impl std::error::Error for FrameError {}

/// One DSU datagram; the CRC32 is computed with its own field zeroed.
pub fn frame_dsu(
    magic: Magic,
    source: SourceId,
    event: Event,
    body: &[u8],
) -> Result<Vec<u8>, FrameError> {
    let too_long = |_overflow| FrameError {
        body_len: body.len(),
    };
    let length = u16::try_from(EVENT_ID_BYTES + body.len()).map_err(too_long)?;
    let mut packet = Vec::with_capacity(HEADER_BYTES + body.len());
    packet.extend_from_slice(magic.bytes());
    packet.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    packet.extend_from_slice(&length.to_le_bytes());
    packet.extend_from_slice(&[0; CRC_RANGE.end - CRC_RANGE.start]); // filled below
    packet.extend_from_slice(&source.0.to_le_bytes());
    packet.extend_from_slice(&event.code().to_le_bytes());
    packet.extend_from_slice(body);
    let Crc32(crc) = Crc32::compute(&packet);
    if let Some(field) = packet.get_mut(CRC_RANGE) {
        field.copy_from_slice(&crc.to_le_bytes());
    }
    Ok(packet)
}

/// The event id of a datagram, if it has a full header with a known one.
pub(crate) fn event_of(packet: &[u8]) -> Option<Event> {
    let field = packet.get(EVENT_RANGE)?;
    Event::from_code(u32::from_le_bytes(field.try_into().ok()?))
}

#[cfg(test)]
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, byte| {
        write!(out, "{byte:02x}").unwrap();
        out
    })
}

#[cfg(test)]
mod tests {
    use crate::wire::frame::{FrameError, Magic, SourceId, frame_dsu, hex};
    use crate::wire::protocol::{Event, SERVER_ID_BASE};

    /// The id of the server whose frame was captured (bound to port 26760).
    const GOLDEN_SOURCE: SourceId = SourceId(SERVER_ID_BASE + 26_760);
    /// The version reply's body: protocol version 1001.
    const VERSION_BODY: [u8; 2] = [0xe9, 0x03];
    /// Golden bytes from `frameDsu` in `src/dsu_server.ts`.
    const GOLDEN_FRAME: &str = "44535553e903060010ba2c2e6629127000001000e903";
    /// One byte more than the 16-bit length field can count with the event id.
    const OVERSIZED_BODY: usize = 65_532;

    #[test]
    fn matches_typescript_frame() {
        let packet = frame_dsu(Magic::Server, GOLDEN_SOURCE, Event::Version, &VERSION_BODY);
        assert_eq!(hex(&packet.unwrap()), GOLDEN_FRAME);
    }

    #[test]
    fn rejects_oversized_body() {
        let body = vec![0; OVERSIZED_BODY];
        assert_eq!(
            frame_dsu(Magic::Server, SourceId(0), Event::Pad, &body),
            Err(FrameError {
                body_len: OVERSIZED_BODY
            })
        );
    }

    #[test]
    fn reads_magic_back() {
        let packet = frame_dsu(Magic::Client, SourceId(0), Event::Pad, &[]).unwrap();
        assert_eq!(Magic::of_packet(&packet), Some(Magic::Client));
        assert_eq!(Magic::of_packet(b"XXXX"), None);
    }
}
