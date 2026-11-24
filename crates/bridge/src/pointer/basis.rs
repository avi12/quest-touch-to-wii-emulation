use crate::math::{
    Mat3, Vec3, WORLD_UP, cross, multiply_transposed, norm, normalize, remove_component,
};

const DEFAULT_UP_HINT_LIMIT: f64 = 0.9;
const MIN_UP_LENGTH: f64 = 0.2;
const FORWARD_HINT: Vec3 = [0.0, 0.0, 1.0];

/// Basis (right, forward, up) in the controller frame for a pointing axis and an "up" hint.
pub(crate) fn make_basis(forward: Vec3, upward: Vec3) -> Mat3 {
    [cross(forward, upward), forward, upward]
}

/// Any axis not parallel to the aim, made perpendicular to it.
pub(crate) fn default_up(forward: Vec3) -> Vec3 {
    let hint = if forward[1].abs() < DEFAULT_UP_HINT_LIMIT {
        WORLD_UP
    } else {
        FORWARD_HINT
    };
    normalize(remove_component(hint, forward))
}

/// The hand is upright at recenter, whatever the grip: `None` when the aim is too near vertical to
/// tell which way is up.
pub(crate) fn upright_basis(rotation: Mat3, aim: Vec3) -> Option<Mat3> {
    let upward = remove_component(multiply_transposed(rotation, WORLD_UP), aim);
    if norm(upward) <= MIN_UP_LENGTH {
        return None;
    }
    Some(make_basis(aim, normalize(upward)))
}
