//! The C layouts and function-table slots of `IVRSystem_023` and `IVRRenderModels_006`.

use std::ffi::{c_char, c_void};
use std::ptr::NonNull;

use crate::device::transform::Matrix34;

/// `TrackedDevicePose_t`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RawPose {
    pub(crate) device_to_absolute: Matrix34,
    pub(crate) velocity: [f32; 3],
    pub(crate) angular_velocity: [f32; 3],
    pub(crate) tracking_result: i32,
    pub(crate) valid: u8,
    pub(crate) connected: u8,
}

/// `VRControllerState_t`: five axes of (x, y).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RawControllerState {
    pub(crate) packet_number: u32,
    pub(crate) button_pressed: u64,
    pub(crate) button_touched: u64,
    pub(crate) axes: [[f32; 2]; 5],
}

/// `RenderModel_ControllerMode_State_t`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RawControllerMode {
    pub(crate) scroll_wheel_visible: u8,
}

/// `RenderModel_ComponentState_t`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct RawComponentState {
    pub(crate) tracking_to_component_render_model: Matrix34,
    pub(crate) tracking_to_component_local: Matrix34,
    pub(crate) properties: u32,
}

// The DLL writes these; a layout drift must fail the build, not read garbage.
const _: () = assert!(size_of::<RawPose>() == 80);
const _: () = assert!(size_of::<RawControllerState>() == 64);
const _: () = assert!(size_of::<RawComponentState>() == 100);

type GetPosesFn = unsafe extern "system" fn(i32, f32, *mut RawPose, u32);
type IndexForRoleFn = unsafe extern "system" fn(i32) -> u32;
type DeviceToI32Fn = unsafe extern "system" fn(u32) -> i32;
type IsConnectedFn = unsafe extern "system" fn(u32) -> bool;
type GetStringFn = unsafe extern "system" fn(u32, i32, *mut c_char, u32, *mut i32) -> u32;
type GetStateFn = unsafe extern "system" fn(u32, *mut RawControllerState, u32) -> bool;
type GetComponentStateFn = unsafe extern "system" fn(
    *const c_char,
    *const c_char,
    *const RawControllerState,
    *const RawControllerMode,
    *mut RawComponentState,
) -> bool;

/// The `IVRSystem` calls the bridge uses (slots in header order).
#[derive(Clone, Copy)]
pub(crate) struct SystemTable {
    pub(crate) get_poses: GetPosesFn,
    pub(crate) index_for_role: IndexForRoleFn,
    pub(crate) role_for_index: DeviceToI32Fn,
    pub(crate) device_class: DeviceToI32Fn,
    pub(crate) is_connected: IsConnectedFn,
    pub(crate) string_property: GetStringFn,
    pub(crate) controller_state: GetStateFn,
}

/// The `IVRRenderModels` calls the bridge uses.
#[derive(Clone, Copy)]
pub(crate) struct RenderModelsTable {
    pub(crate) component_state: GetComponentStateFn,
}

/// `IVRSystem_023` slots, in header order.
const GET_POSES_SLOT: usize = 11;
const INDEX_FOR_ROLE_SLOT: usize = 17;
const ROLE_FOR_INDEX_SLOT: usize = 18;
const DEVICE_CLASS_SLOT: usize = 19;
const IS_CONNECTED_SLOT: usize = 20;
const STRING_PROPERTY_SLOT: usize = 27;
const CONTROLLER_STATE_SLOT: usize = 34;
/// `IVRRenderModels_006` slot of `GetComponentState`.
const COMPONENT_STATE_SLOT: usize = 10;

/// Reads one slot of a function table.
///
/// # Safety
/// `table` must point to a function table with more than `index` slots, and `F` must be
/// `Option` of the slot's exact function-pointer type.
unsafe fn slot<F>(table: NonNull<c_void>, index: usize) -> F {
    // SAFETY: the caller guarantees the slot exists and holds an `F`.
    unsafe { table.cast::<F>().add(index).read() }
}

impl SystemTable {
    /// Binds the table, or `None` when a slot is null.
    ///
    /// # Safety
    /// `table` must be the `FnTable:IVRSystem_023` pointer from `VR_GetGenericInterface`.
    pub(crate) unsafe fn load(table: NonNull<c_void>) -> Option<Self> {
        // SAFETY: IVRSystem_023 has more than 34 slots, each typed as in openvr_capi.h.
        unsafe {
            Some(Self {
                get_poses: slot::<Option<GetPosesFn>>(table, GET_POSES_SLOT)?,
                index_for_role: slot::<Option<IndexForRoleFn>>(table, INDEX_FOR_ROLE_SLOT)?,
                role_for_index: slot::<Option<DeviceToI32Fn>>(table, ROLE_FOR_INDEX_SLOT)?,
                device_class: slot::<Option<DeviceToI32Fn>>(table, DEVICE_CLASS_SLOT)?,
                is_connected: slot::<Option<IsConnectedFn>>(table, IS_CONNECTED_SLOT)?,
                string_property: slot::<Option<GetStringFn>>(table, STRING_PROPERTY_SLOT)?,
                controller_state: slot::<Option<GetStateFn>>(table, CONTROLLER_STATE_SLOT)?,
            })
        }
    }
}

impl RenderModelsTable {
    /// Binds the table, or `None` when a slot is null.
    ///
    /// # Safety
    /// `table` must be the `FnTable:IVRRenderModels_006` pointer from `VR_GetGenericInterface`.
    pub(crate) unsafe fn load(table: NonNull<c_void>) -> Option<Self> {
        // SAFETY: IVRRenderModels_006 has more than 10 slots, each typed as in openvr_capi.h.
        let component_state =
            unsafe { slot::<Option<GetComponentStateFn>>(table, COMPONENT_STATE_SLOT)? };
        Some(Self { component_state })
    }
}

/// A buffer or struct length as the `u32` the C API takes.
pub(crate) fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}
