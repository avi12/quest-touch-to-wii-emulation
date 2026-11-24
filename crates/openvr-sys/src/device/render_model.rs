//! Render-model components, such as where the "tip" of a controller sits.

use std::ffi::{CStr, CString};

use crate::device::controller::read_raw;
use crate::device::index::DeviceIndex;
use crate::device::transform::ComponentTransform;
use crate::ffi::table::{RawComponentState, RawControllerMode, RenderModelsTable, SystemTable};

/// A named part of a controller's render model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Component {
    /// `tip`: the aim ray's origin and direction.
    Tip,
    /// `base`: the bottom of the controller.
    Base,
    /// `handgrip`: where the hand holds it.
    Handgrip,
}

impl Component {
    /// Every component, in declaration order.
    pub const ALL: [Self; 3] = [Self::Tip, Self::Base, Self::Handgrip];

    /// The component's name in the render model.
    pub const fn name(self) -> &'static CStr {
        match self {
            Self::Tip => c"tip",
            Self::Base => c"base",
            Self::Handgrip => c"handgrip",
        }
    }
}

/// Which device, model and component to look up.
pub(crate) struct ComponentQuery<'model> {
    pub(crate) index: DeviceIndex,
    pub(crate) model: &'model str,
    pub(crate) component: Component,
}

/// Where a component sits relative to the controller's tracked pose, or `None` when it has
/// none (or the model name contains a NUL).
pub(crate) fn component_transform(
    system: &SystemTable,
    models: &RenderModelsTable,
    query: &ComponentQuery<'_>,
) -> Option<ComponentTransform> {
    let state = read_raw(system, query.index)?;
    let model = CString::new(query.model).ok()?;
    let mode = RawControllerMode::default();
    let mut out = RawComponentState::default();
    // SAFETY: both names are NUL-terminated, and every struct pointer is valid for the call.
    let is_found = unsafe {
        (models.component_state)(
            model.as_ptr(),
            query.component.name().as_ptr(),
            &raw const state,
            &raw const mode,
            &raw mut out,
        )
    };
    is_found.then(|| ComponentTransform::from_matrix(&out.tracking_to_component_local))
}
