//! The safe `OpenVr` handle: one initialised runtime plus the interface tables it calls.

use std::ffi::CStr;

use crate::controller::{ControllerState, read_raw};
use crate::error::OpenVrError;
use crate::loader::{Runtime, interface_error};
use crate::pose::Poses;
use crate::property::{self, StringProperty};
use crate::render_model::{ComponentQuery, component_transform};
use crate::table::{RenderModelsTable, SystemTable};
use crate::transform::ComponentTransform;

const SYSTEM_INTERFACE: &CStr = c"FnTable:IVRSystem_023";
const RENDER_MODELS_INTERFACE: &CStr = c"FnTable:IVRRenderModels_006";
/// `k_unTrackedDeviceIndexInvalid`.
const INVALID_DEVICE_INDEX: u32 = 0xffff_ffff;

/// Which hand a controller is held in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    /// `TrackedControllerRole_LeftHand`.
    Left,
    /// `TrackedControllerRole_RightHand`.
    Right,
}

impl Hand {
    /// The `ETrackedControllerRole` value.
    fn role(self) -> i32 {
        match self {
            Self::Left => 1,
            Self::Right => 2,
        }
    }
}

/// `OpenVR` initialised as a background application; shut down when dropped.
pub struct OpenVr {
    system: SystemTable,
    render_models: RenderModelsTable,
    // Last, so the tables are never used after the runtime shuts down.
    _runtime: Runtime,
}

impl OpenVr {
    /// Loads `openvr_api.dll` and initialises `OpenVR`. Fails while `SteamVR` is not running.
    pub fn new() -> Result<Self, OpenVrError> {
        let runtime = Runtime::init()?;
        let system_table = runtime.table(SYSTEM_INTERFACE)?;
        // SAFETY: the pointer is the IVRSystem_023 function table.
        let system = unsafe { SystemTable::load(system_table) }
            .ok_or_else(|| interface_error(SYSTEM_INTERFACE, 0))?;
        let models_table = runtime.table(RENDER_MODELS_INTERFACE)?;
        // SAFETY: the pointer is the IVRRenderModels_006 function table.
        let render_models = unsafe { RenderModelsTable::load(models_table) }
            .ok_or_else(|| interface_error(RENDER_MODELS_INTERFACE, 0))?;
        Ok(Self {
            system,
            render_models,
            _runtime: runtime,
        })
    }

    /// Snapshot of every device's pose; each one is decoded only when asked for.
    pub fn poses(&self) -> Poses {
        Poses::fetch(&self.system)
    }

    /// The device index of the controller in `hand`, or `None` when there is none.
    pub fn controller_index(&self, hand: Hand) -> Option<u32> {
        // SAFETY: the call takes a plain role value.
        let index = unsafe { (self.system.index_for_role)(hand.role()) };
        (index != INVALID_DEVICE_INDEX).then_some(index)
    }

    /// Whether device `index` is connected.
    pub fn is_connected(&self, index: u32) -> bool {
        // SAFETY: the call takes a plain device index.
        unsafe { (self.system.is_connected)(index) }
    }

    /// The raw `ETrackedDeviceClass` of device `index` (0 = invalid, 2 = controller, ...).
    pub fn device_class(&self, index: u32) -> i32 {
        // SAFETY: the call takes a plain device index.
        unsafe { (self.system.device_class)(index) }
    }

    /// 0 = none, 1 = left hand, 2 = right hand, as in `SteamVR`'s `ETrackedControllerRole`.
    pub fn controller_role(&self, index: u32) -> i32 {
        // SAFETY: the call takes a plain device index.
        unsafe { (self.system.role_for_index)(index) }
    }

    /// A string property of one device (for example `PROPERTY.render_model_name`), or "" when
    /// it has none.
    pub fn string_property(&self, index: u32, property: StringProperty) -> String {
        property::read(&self.system, index, property)
    }

    /// Sticks, triggers and buttons of one controller, or `None` when `SteamVR` has no state.
    pub fn controller_state(&self, index: u32) -> Option<ControllerState> {
        read_raw(&self.system, index).map(|raw| ControllerState::from(&raw))
    }

    /// Where a render-model component (for example "tip") sits relative to the controller's
    /// tracked pose, or `None` when it has none.
    pub fn component_transform(
        &self,
        index: u32,
        model: &str,
        component: &str,
    ) -> Option<ComponentTransform> {
        let query = ComponentQuery {
            index,
            model,
            component,
        };
        component_transform(&self.system, &self.render_models, &query)
    }
}
