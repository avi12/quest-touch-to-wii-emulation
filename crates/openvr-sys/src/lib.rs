//! Safe wrapper over `SteamVR`'s `openvr_api.dll`: poses, controller state, render-model components
//!
//! `OpenVR` is reached through its C "flat" API: the DLL is loaded at runtime and each interface
//! is called through its function table, so no `SteamVR` SDK is needed to build.

mod controller;
mod error;
mod loader;
mod openvr;
mod pose;
mod property;
mod render_model;
mod table;
mod transform;

pub use controller::{Button, ControllerState};
pub use error::OpenVrError;
pub use openvr::{Hand, OpenVr};
pub use pose::{DevicePose, MAX_DEVICES, Poses};
pub use property::{PROPERTY, Properties, StringProperty};
pub use transform::{ComponentTransform, Mat3, Vec3};
