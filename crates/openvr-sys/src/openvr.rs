//! The safe `OpenVr` handle: one initialised runtime plus the interface tables it calls.

use std::ffi::CStr;

use crate::device::controller::{ControllerState, read_raw};
use crate::device::index::DeviceIndex;
use crate::device::kind::{ControllerRole, DeviceClass, Hand};
use crate::device::pose::Poses;
use crate::device::property::{self, StringProperty};
use crate::device::render_model::{Component, ComponentQuery, component_transform};
use crate::device::transform::ComponentTransform;
use crate::error::OpenVrError;
use crate::ffi::loader::{Runtime, interface_error};
use crate::ffi::table::{RenderModelsTable, SystemTable};

const SYSTEM_INTERFACE: &CStr = c"FnTable:IVRSystem_023";
const RENDER_MODELS_INTERFACE: &CStr = c"FnTable:IVRRenderModels_006";

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
    pub fn controller_index(&self, hand: Hand) -> Option<DeviceIndex> {
        // SAFETY: the call takes a plain role value.
        let raw = unsafe { (self.system.index_for_role)(hand.role().code()) };
        // `k_unTrackedDeviceIndexInvalid` is out of range, so it maps to `None`.
        DeviceIndex::new(raw)
    }

    /// Whether device `index` is connected.
    pub fn is_connected(&self, index: DeviceIndex) -> bool {
        // SAFETY: the call takes a plain device index.
        unsafe { (self.system.is_connected)(index.get()) }
    }

    /// What kind of device sits at `index`; `None` for a class this binding does not know.
    pub fn device_class(&self, index: DeviceIndex) -> Option<DeviceClass> {
        // SAFETY: the call takes a plain device index.
        let code = unsafe { (self.system.device_class)(index.get()) };
        DeviceClass::from_code(code)
    }

    /// The controller role of device `index`; `None` for a role this binding does not know.
    pub fn controller_role(&self, index: DeviceIndex) -> Option<ControllerRole> {
        // SAFETY: the call takes a plain device index.
        let code = unsafe { (self.system.role_for_index)(index.get()) };
        ControllerRole::from_code(code)
    }

    /// A string property of one device, or "" when it has none.
    pub fn string_property(&self, index: DeviceIndex, property: StringProperty) -> String {
        property::read(&self.system, index, property)
    }

    /// Sticks, triggers and buttons of one controller, or `None` when `SteamVR` has no state.
    pub fn controller_state(&self, index: DeviceIndex) -> Option<ControllerState> {
        read_raw(&self.system, index).map(|raw| ControllerState::from(&raw))
    }

    /// Where a render-model component sits relative to the controller's tracked pose, or `None`
    /// when it has none. `model` is the device's [`StringProperty::RenderModelName`].
    pub fn component_transform(
        &self,
        index: DeviceIndex,
        model: &str,
        component: Component,
    ) -> Option<ComponentTransform> {
        let query = ComponentQuery {
            index,
            model,
            component,
        };
        component_transform(&self.system, &self.render_models, &query)
    }
}
