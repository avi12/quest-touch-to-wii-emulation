//! DSU datagram framing: 16 header bytes, CRC32 over the whole packet, event id, body.

use std::fmt;
use std::ops::Range;

use crate::protocol::{Event, PROTOCOL_VERSION};

/// Header bytes including the event id.
pub(crate) const HEADER_BYTES: usize = 20;
const EVENT_ID_BYTES: usize = 4;
const CRC_RANGE: Range<usize> = 8..12;

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
}

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
    source_id: u32,
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
    packet.extend_from_slice(&[0; 4]); // CRC, filled below
    packet.extend_from_slice(&source_id.to_le_bytes());
    packet.extend_from_slice(&event.code().to_le_bytes());
    packet.extend_from_slice(body);
    let crc = crc32fast::hash(&packet);
    if let Some(field) = packet.get_mut(CRC_RANGE) {
        field.copy_from_slice(&crc.to_le_bytes());
    }
    Ok(packet)
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
    use super::{FrameError, Magic, frame_dsu, hex};
    use crate::protocol::{Event, SERVER_ID_BASE};

    /// Golden bytes from `frameDsu` in `src/dsu_server.ts`.
    #[test]
    fn matches_typescript_frame() {
        let packet = frame_dsu(
            Magic::Server,
            SERVER_ID_BASE + 26760,
            Event::Version,
            &[0xe9, 0x03],
        );
        assert_eq!(
            hex(&packet.unwrap()),
            "44535553e903060010ba2c2e6629127000001000e903"
        );
    }

    #[test]
    fn rejects_oversized_body() {
        let body = vec![0; 65_533];
        assert_eq!(
            frame_dsu(Magic::Server, 0, Event::Pad, &body),
            Err(FrameError { body_len: 65_533 })
        );
    }
}
