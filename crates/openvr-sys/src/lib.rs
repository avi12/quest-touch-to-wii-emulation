//! Safe wrapper over `SteamVR`'s `openvr_api.dll`: poses, controller state, render-model components
//!
//! `OpenVR` is reached through its C "flat" API: the DLL is loaded at runtime and each interface
//! is called through its function table, so no `SteamVR` SDK is needed to build.

mod device;
mod error;
mod ffi;
mod openvr;

pub use device::controller::{Button, ControllerState};
pub use device::pose::{DevicePose, MAX_DEVICES, Poses};
pub use device::property::{PROPERTY, Properties, StringProperty};
pub use device::transform::{ComponentTransform, Mat3, Vec3};
pub use error::OpenVrError;
pub use openvr::{Hand, OpenVr};
