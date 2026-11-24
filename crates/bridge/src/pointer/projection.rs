use super::PointerPosition;
use super::screen::Screen;
use crate::math::{OneEuro, Vec3, degrees_to_radians};

const YAW_RANGE: f64 = degrees_to_radians(28.0); // controller swing (each way) that spans the whole screen width
const PITCH_RANGE: f64 = degrees_to_radians(30.0);
const PITCH_BIAS: f64 = 0.0; // keep 0: a recenter must put the hand exactly in the middle of the screen
const EDGE_THRESHOLD: f64 = 0.999; // pointer value that counts as pinned at the screen edge
const POINTER_LIMIT: f64 = 0.95; // Dolphin drops each IR dot that leaves its camera frame; at +-1 one dot is already on the edge
/// Cosine between the aim and the screen's normal below which the aim is ignored (60 degrees off);
/// nearer the plane the projection blows up to a corner.
pub(crate) const MIN_DEPTH: f64 = 0.5;
const CENTER: PointerPosition = PointerPosition { x: 0.0, y: 0.0 };

/// Pointer position for this frame, plus how squarely the aim faces the screen.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Projection {
    pub(crate) position: PointerPosition,
    pub(crate) facing: f64,
}

/// Projects the aim onto the screen and smooths the result per axis.
#[derive(Clone, Debug, Default)]
pub(crate) struct Projector {
    filter_x: OneEuro,
    filter_y: OneEuro,
}

fn is_at_edge(filter: &OneEuro) -> bool {
    filter.value().unwrap_or_default().abs() >= EDGE_THRESHOLD
}

impl Projector {
    pub(crate) fn reset(&mut self) {
        self.filter_x.reset();
        self.filter_y.reset();
    }

    /// Whether the smoothed pointer sits on a screen edge.
    pub(crate) fn is_pinned(&self) -> bool {
        is_at_edge(&self.filter_x) || is_at_edge(&self.filter_y)
    }

    /// Where the aim ray hits a flat screen.
    pub(crate) fn project(
        &mut self,
        screen: &Screen,
        direction: Vec3,
        delta_seconds: f64,
    ) -> Projection {
        let depth = screen.depth(direction);
        let is_behind_screen = depth < MIN_DEPTH; // aimed away from the screen: no hit point, so the pointer stays put
        if is_behind_screen {
            // placeholder: the caller sees is_aimed_away and keeps its previous pointer
            return Projection {
                position: CENTER,
                facing: depth,
            };
        }
        let [raw_x, raw_y] = screen.hit(direction, depth);
        let filtered_x = self.filter_x.filter(raw_x / YAW_RANGE.tan(), delta_seconds);
        let filtered_y = self
            .filter_y
            .filter(raw_y / PITCH_RANGE.tan(), delta_seconds);
        Projection {
            position: PointerPosition {
                x: filtered_x.clamp(-POINTER_LIMIT, POINTER_LIMIT),
                y: (filtered_y + PITCH_BIAS).clamp(-POINTER_LIMIT, POINTER_LIMIT),
            },
            facing: depth,
        }
    }
}
