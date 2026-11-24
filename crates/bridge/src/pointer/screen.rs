use crate::math::{Vec3, WORLD_UP, cross, dot, norm, normalize};

const USABLE_AIM_LIMIT: f64 = 0.95; // aiming straight up/down has no usable screen plane

/// Whether an aim direction is far enough from vertical to place a screen in front of it.
pub(crate) fn is_aim_usable(direction: Vec3) -> bool {
    direction[1].abs() < USABLE_AIM_LIMIT
}

/// A flat virtual screen in the world, facing the player.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Screen {
    normal: Vec3,
    right: Vec3,
    upward: Vec3,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            normal: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            upward: WORLD_UP,
        }
    }
}

impl Screen {
    /// The screen is where you aim at recenter, pitch included: `None` when aimed near vertical.
    pub(crate) fn facing(direction: Vec3) -> Option<Self> {
        let normal = normalize(direction);
        if !is_aim_usable(normal) {
            return None;
        }
        let right = normalize(cross(normal, WORLD_UP));
        Some(Self {
            normal,
            right,
            upward: cross(right, normal),
        })
    }

    /// Cosine between `direction` and the screen's normal; low or negative is aimed away.
    pub(crate) fn depth(&self, direction: Vec3) -> f64 {
        dot(direction, self.normal) / norm(direction)
    }

    /// Where the aim ray hits the screen plane, as tangents of the angle off its centre.
    pub(crate) fn hit(&self, direction: Vec3, depth: f64) -> [f64; 2] {
        let length = norm(direction);
        [
            dot(direction, self.right) / length / depth,
            dot(direction, self.upward) / length / depth,
        ]
    }
}
