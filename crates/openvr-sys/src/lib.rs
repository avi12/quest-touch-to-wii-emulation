//! Safe wrapper over `SteamVR`'s `openvr_api.dll`: poses, controller state, render-model components
//!
//! `OpenVR` is reached through its C "flat" API: the DLL is loaded at runtime and each interface
//! is called through its function table, so no `SteamVR` SDK is needed to build. Vectors and
//! rotations are `geometry::{Vec3, Mat3}`.

mod device;
mod error;
mod ffi;
mod openvr;

pub use device::controller::{Button, ControllerState};
pub use device::index::DeviceIndex;
pub use device::kind::{ControllerRole, DeviceClass, Hand};
pub use device::pose::{DevicePose, Poses, TrackingResult};
pub use device::property::StringProperty;
pub use device::render_model::Component;
pub use device::transform::ComponentTransform;
pub use error::OpenVrError;
pub use openvr::OpenVr;
