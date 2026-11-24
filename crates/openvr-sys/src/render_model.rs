//! Render-model components, such as where the "tip" of a controller sits.

use std::ffi::CString;

use crate::controller::read_raw;
use crate::table::{RawComponentState, RawControllerMode, RenderModelsTable, SystemTable};
use crate::transform::ComponentTransform;

/// Which device, model and component to look up.
pub(crate) struct ComponentQuery<'name> {
    pub(crate) index: u32,
    pub(crate) model: &'name str,
    pub(crate) component: &'name str,
}

/// Where a component sits relative to the controller's tracked pose, or `None` when it has
/// none (or the names contain a NUL).
pub(crate) fn component_transform(
    system: &SystemTable,
    models: &RenderModelsTable,
    query: &ComponentQuery<'_>,
) -> Option<ComponentTransform> {
    let state = read_raw(system, query.index)?;
    let model = CString::new(query.model).ok()?;
    let component = CString::new(query.component).ok()?;
    let mode = RawControllerMode::default();
    let mut out = RawComponentState::default();
    // SAFETY: both names are NUL-terminated, and every struct pointer is valid for the call.
    let found = unsafe {
        (models.component_state)(
            model.as_ptr(),
            component.as_ptr(),
            &raw const state,
            &raw const mode,
            &raw mut out,
        )
    };
    found.then(|| ComponentTransform::from_matrix(&out.tracking_to_component_local))
}
