//! Why `OpenVR` could not be brought up.

use std::error::Error;
use std::ffi::CStr;
use std::fmt::{self, Display, Formatter};
use std::io;

/// A failure to load or initialise `OpenVR`.
#[derive(Debug)]
pub enum OpenVrError {
    /// `openvr_api.dll` is not under any known Steam directory.
    DllNotFound,
    /// The DLL exists but Windows could not load it.
    Load(io::Error),
    /// The DLL lacks an entry point the binding calls.
    MissingEntry(&'static CStr),
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
            Self::MissingEntry(name) => {
                write!(
                    formatter,
                    "openvr_api.dll has no {}",
                    name.to_string_lossy()
                )
            }
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
            Self::DllNotFound
            | Self::MissingEntry(_)
            | Self::Init { .. }
            | Self::Interface { .. } => None,
        }
    }
}
