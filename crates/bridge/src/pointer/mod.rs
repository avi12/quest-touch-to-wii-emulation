//! Absolute IR pointer from where the controller aims, with live-selectable pointing axes.

mod basis;
mod candidates;
mod projection;
mod recenter;
mod screen;
mod sweep;
#[cfg(test)]
mod tests;

pub use candidates::{CANDIDATES, Candidate, CandidateStep, DEFAULT_CANDIDATE_INDEX};
pub use sweep::sweep_pointer;

use basis::{default_up, make_basis, upright_basis};
use candidates::step_candidate;
use projection::{MIN_DEPTH, Projector};
use recenter::{EdgeHold, EdgeSample};
use screen::Screen;

use std::time::Duration;

use crate::math::{Mat3, Vec3, multiply_matrix, normalize};

/// Pointer position, each axis in -1..1 across the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointerPosition {
    pub x: f64,
    pub y: f64,
}

/// One frame's input to [`Pointer::update`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    /// Controller-to-world rotation, row-major.
    pub rotation: Mat3,
    /// Thumbstick click: a fresh press recenters.
    pub is_click_pressed: bool,
    /// Time since the previous update.
    pub delta: Duration,
    /// Monotonic clock, for the edge-hold auto-recenter.
    pub now: Duration,
}

/// One frame's output of [`Pointer::update`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerUpdate {
    pub position: PointerPosition,
    /// This update recentred the pointer (first sample, stick click, or edge hold).
    pub has_recentered: bool,
    /// Aimed too far from the screen to have a hit point; the caller keeps its last pointer.
    pub is_aimed_away: bool,
    /// Cosine between the aim and the screen's normal; low or negative means aimed away from the
    /// screen.
    pub facing: f64,
    /// Where the aim axis points in the world.
    pub direction: Vec3,
}

/// Absolute pointer from where the controller aims. Recenters on the first sample, on a thumbstick
/// click, or when pinned at a screen edge for a while.
#[derive(Clone, Debug)]
pub struct Pointer {
    /// Right, forward, up (in the controller frame).
    basis: Mat3,
    /// Pointing axis, chosen live with the Ctrl+Alt+[ ] selector; grip-dependent.
    aim: Vec3,
    candidate_index: usize,
    screen: Screen,
    is_recenter_needed: bool,
    /// The thumbstick click was held on the previous update.
    is_click_held: bool,
    edge_hold: EdgeHold,
    projector: Projector,
}

impl Default for Pointer {
    fn default() -> Self {
        let aim = normalize(CANDIDATES[DEFAULT_CANDIDATE_INDEX].axis);
        Self {
            basis: make_basis(aim, default_up(aim)),
            aim,
            candidate_index: DEFAULT_CANDIDATE_INDEX,
            screen: Screen::default(),
            is_recenter_needed: true,
            is_click_held: false,
            edge_hold: EdgeHold::default(),
            projector: Projector::default(),
        }
    }
}

impl Pointer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Right, forward, up of the Wiimote in the controller frame, for `motion::to_sensors`.
    #[must_use]
    pub fn basis(&self) -> Mat3 {
        self.basis
    }

    #[must_use]
    pub fn candidate_index(&self) -> usize {
        self.candidate_index
    }

    /// Switches the pointing axis one candidate on or back and recenters on the next update.
    /// Returns the candidate now in use, for the caller to log.
    pub fn cycle_candidate(&mut self, step: CandidateStep) -> Candidate {
        self.candidate_index = step_candidate(self.candidate_index, step);
        let candidate = CANDIDATES
            .get(self.candidate_index)
            .copied()
            .unwrap_or(Candidate {
                name: "",
                axis: self.aim,
            });
        self.aim = normalize(candidate.axis);
        self.is_recenter_needed = true;
        candidate
    }

    pub fn update(&mut self, input: &PointerInput) -> PointerUpdate {
        let direction = multiply_matrix(input.rotation, self.aim);
        let edge = EdgeSample {
            direction,
            delta: input.delta,
            now: input.now,
            is_pinned: self.projector.is_pinned(),
        };
        if self.edge_hold.is_pinned_and_still(&edge) {
            self.is_recenter_needed = true;
        }
        let is_fresh_click = input.is_click_pressed && !self.is_click_held;
        let has_recentered = self.is_recenter_needed || is_fresh_click;
        self.is_click_held = input.is_click_pressed;
        if has_recentered {
            self.recenter(input.rotation, direction);
        }
        let projection = self.projector.project(&self.screen, direction, input.delta);
        PointerUpdate {
            position: projection.position,
            has_recentered,
            is_aimed_away: projection.facing < MIN_DEPTH,
            facing: projection.facing,
            direction,
        }
    }

    fn recenter(&mut self, rotation: Mat3, direction: Vec3) {
        self.is_recenter_needed = false;
        self.projector.reset();
        if let Some(screen) = Screen::facing(direction) {
            self.screen = screen;
        }
        if let Some(basis) = upright_basis(rotation, self.aim) {
            self.basis = basis;
        }
    }
}
