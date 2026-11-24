//! Why `OpenVR` could not be brought up.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A failure to load or initialise `OpenVR`.
#[derive(Debug)]
pub enum OpenVrError {
    /// `openvr_api.dll` is not under any known Steam directory.
    DllNotFound,
    /// The DLL exists but could not be loaded, or lacks an entry point.
    Load(libloading::Error),
    /// `VR_InitInternal2` failed (for example, `SteamVR` is not running).
    Init {
        /// `SteamVR`'s name for the error, such as `VRInitError_Init_NoServerForBackgroundApp`.
        symbol: String,
        /// The raw `EVRInitError` code.
        code: i32,
    },
    /// An interface's function table is missing or has a null slot.
    Interface {
        /// The interface version string asked for.
        name: String,
        /// The raw `EVRInitError` code (0 when the table had a null slot).
        code: i32,
    },
}

impl Display for OpenVrError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DllNotFound => {
                write!(formatter, "openvr_api.dll not found; is SteamVR installed?")
            }
            Self::Load(error) => write!(formatter, "openvr_api.dll failed to load: {error}"),
            Self::Init { symbol, code } => {
                write!(formatter, "OpenVR init failed: {symbol} ({code})")
            }
            Self::Interface { name, code } => {
                write!(formatter, "OpenVR {name} unavailable ({code})")
            }
        }
    }
}

impl Error for OpenVrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::DllNotFound | Self::Init { .. } | Self::Interface { .. } => None,
        }
    }
}

impl From<libloading::Error> for OpenVrError {
    fn from(error: libloading::Error) -> Self {
        Self::Load(error)
    }
}
