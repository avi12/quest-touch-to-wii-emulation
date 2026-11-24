//! `SteamVR` access shared by the tools that read the controller directly.

use openvr_sys::{DeviceIndex, Hand, OpenVr};

/// Initialises `SteamVR`, or a message saying why it failed.
pub fn open() -> Result<OpenVr, String> {
    OpenVr::new().map_err(|error| format!("SteamVR unavailable: {error}"))
}

/// The right-hand controller's device index, or the message to show when `SteamVR` has none.
pub fn right_controller(openvr: &OpenVr) -> Result<DeviceIndex, String> {
    openvr.controller_index(Hand::Right).ok_or_else(|| {
        "no right-hand controller: SteamVR does not see it (is it awake and tracked?)".to_owned()
    })
}
