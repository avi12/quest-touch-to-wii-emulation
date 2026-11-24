//! Why the bridge could not start.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;

use openvr_sys::OpenVrError;

/// A start-up failure; the loop itself only logs and carries on.
#[derive(Debug)]
pub enum BridgeError {
    /// `SteamVR` is not running, or the headset is not streaming yet.
    OpenVr(OpenVrError),
    /// The DSU port is taken (another bridge on it?).
    Bind { port: u16, source: io::Error },
    /// The telemetry file could not be opened for appending.
    Telemetry { path: PathBuf, source: io::Error },
}

impl Display for BridgeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenVr(error) => write!(
                formatter,
                "{error}; start SteamVR with the headset connected, then run the bridge again"
            ),
            Self::Bind { port, source } => {
                write!(formatter, "cannot serve DSU on UDP {port}: {source}")
            }
            Self::Telemetry { path, source } => {
                write!(
                    formatter,
                    "cannot open telemetry file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for BridgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenVr(error) => Some(error),
            Self::Bind { source, .. } | Self::Telemetry { source, .. } => Some(source),
        }
    }
}
