//! UDP socket that asks a DSU server for pad data, one request per reply.

use std::fmt;
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use dsu::{Event, FrameError, Magic, SourceId, frame_dsu};

use dsu::Port;

/// The bridge listens on the loopback interface.
pub const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;
const CLIENT_ID: SourceId = SourceId(1234);
const REPLY_TIMEOUT: Duration = Duration::from_secs(1);
/// Actions, slot, MAC (6 bytes), all zero: "any pad".
const REQUEST_BODY: [u8; 9] = [0; 9];
const MAX_DATAGRAM_BYTES: usize = 2048;

/// Why the probe could not start.
#[derive(Debug)]
pub enum ProbeError {
    Socket(io::Error),
    Frame(FrameError),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Socket(error) => write!(formatter, "UDP socket: {error}"),
            Self::Frame(error) => write!(formatter, "DSU request: {error}"),
        }
    }
}

impl std::error::Error for ProbeError {}

/// A DSU client bound to an ephemeral port, talking to one server.
#[derive(Debug)]
pub struct DsuClient {
    socket: UdpSocket,
    server: SocketAddr,
    request: Vec<u8>,
}

impl DsuClient {
    /// A client for the DSU server on `LOOPBACK:port`.
    pub fn new(port: Port) -> Result<Self, ProbeError> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(ProbeError::Socket)?;
        socket
            .set_read_timeout(Some(REPLY_TIMEOUT))
            .map_err(ProbeError::Socket)?;
        let request = frame_dsu(Magic::Client, CLIENT_ID, Event::Pad, &REQUEST_BODY)
            .map_err(ProbeError::Frame)?;
        Ok(Self {
            socket,
            server: SocketAddr::from((LOOPBACK, port.get())),
            request,
        })
    }

    /// Requests pad data and returns the next datagram, or `None` when none came within a
    /// second.
    pub fn request_pad(&self) -> Option<Vec<u8>> {
        let mut buffer = [0; MAX_DATAGRAM_BYTES];
        let received = self
            .socket
            .send_to(&self.request, self.server)
            .and_then(|_sent| self.socket.recv(&mut buffer));
        match received {
            Ok(length) => buffer.get(..length).map(<[u8]>::to_vec),
            Err(error) => {
                // Windows reports an unanswered request at once ("connection reset"); wait out
                // the reply timeout anyway so the probe paces itself as when nothing replies.
                let is_timeout =
                    matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut);
                if !is_timeout {
                    std::thread::sleep(REPLY_TIMEOUT);
                }
                None
            }
        }
    }
}
