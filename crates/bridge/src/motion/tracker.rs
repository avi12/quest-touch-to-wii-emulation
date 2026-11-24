use std::collections::VecDeque;

use super::{Motion, PoseSample};
use crate::math::{Vec3, add, degrees_to_radians, norm, scale, subtract};

const MAX_ANGULAR_RATE: f64 = degrees_to_radians(1200.0); // faster than this is a tracking jump, not real motion
const ACCELERATION_WINDOW_SECONDS: f64 = 0.015; // velocity change is measured over at least this long
const ACCELERATION_SMOOTHING: f64 = 0.35;
const NO_MOTION: Vec3 = [0.0; 3];

#[derive(Clone, Copy, Debug)]
struct VelocitySample {
    sampled_at: f64,
    velocity: Vec3,
}

/// Turns `SteamVR`'s own velocity and angular velocity into a smoothed world-frame acceleration and
/// angular rate. Acceleration is the velocity change over a window, not a step: poses are
/// extrapolated afresh on every call (about 750 a second), so differencing consecutive ones
/// amplified noise into spikes of 20 g.
#[derive(Clone, Debug)]
pub struct MotionTracker {
    history: VecDeque<VelocitySample>,
    linear: Vec3,
    angular: Vec3,
    smoothed: Vec3,
}

impl Default for MotionTracker {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            linear: NO_MOTION,
            angular: NO_MOTION,
            smoothed: NO_MOTION,
        }
    }
}

impl MotionTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, pose: &PoseSample) -> Motion {
        self.track_velocity(pose.velocity, pose.timestamp_seconds);
        let is_real_rotation = norm(pose.angular_velocity) <= MAX_ANGULAR_RATE; // faster is a tracking jump
        if is_real_rotation {
            self.angular = pose.angular_velocity;
        }
        self.smoothed = add(
            scale(self.smoothed, ACCELERATION_SMOOTHING),
            scale(self.linear, 1.0 - ACCELERATION_SMOOTHING),
        );
        Motion {
            acceleration: self.smoothed,
            angular_rate: self.angular,
        }
    }

    fn track_velocity(&mut self, velocity: Vec3, now: f64) {
        self.history.push_back(VelocitySample {
            sampled_at: now,
            velocity,
        });
        while self
            .history
            .get(1)
            .map_or(f64::INFINITY, |sample| sample.sampled_at)
            <= now - ACCELERATION_WINDOW_SECONDS
        {
            self.history.pop_front();
        }
        let Some(anchor) = self.history.front() else {
            return;
        };
        let seconds = now - anchor.sampled_at;
        let has_baseline = seconds >= ACCELERATION_WINDOW_SECONDS / 2.0;
        if !has_baseline {
            return;
        }
        self.linear = scale(subtract(velocity, anchor.velocity), 1.0 / seconds);
    }
}
