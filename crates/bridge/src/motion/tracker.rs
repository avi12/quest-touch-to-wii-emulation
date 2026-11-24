use std::collections::VecDeque;
use std::time::Duration;

use crate::math::{
    Degrees, MetresPerSecondSquared, Radians, RadiansPerSecond, Vec3, add, norm, scale, subtract,
};
use crate::motion::{Motion, PoseSample};

const MAX_ANGULAR_RATE: Radians = Degrees(1200.0).to_radians(); // per second; faster is a tracking jump, not real motion
const ACCELERATION_WINDOW: Duration = Duration::from_millis(15); // velocity change is measured over at least this long
const ACCELERATION_SMOOTHING: f64 = 0.35;
const NO_MOTION: Vec3 = [0.0; 3];

#[derive(Clone, Copy, Debug)]
struct VelocitySample {
    /// Monotonic clock, seconds.
    sampled_at: f64,
    /// m/s.
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
        self.track_velocity(pose.velocity.0, pose.timestamp.as_secs_f64());
        let RadiansPerSecond(angular_velocity) = pose.angular_velocity;
        let is_real_rotation = norm(angular_velocity) <= MAX_ANGULAR_RATE.0; // faster is a tracking jump
        if is_real_rotation {
            self.angular = angular_velocity;
        }
        self.smoothed = add(
            scale(self.smoothed, ACCELERATION_SMOOTHING),
            scale(self.linear, 1.0 - ACCELERATION_SMOOTHING),
        );
        Motion {
            acceleration: MetresPerSecondSquared(self.smoothed),
            angular_rate: RadiansPerSecond(self.angular),
        }
    }

    /// Whether the second-oldest sample is already a full window old, so the oldest can go.
    fn is_oldest_stale(&self, now: f64) -> bool {
        self.history
            .get(1)
            .is_some_and(|sample| sample.sampled_at <= now - ACCELERATION_WINDOW.as_secs_f64())
    }

    fn track_velocity(&mut self, velocity: Vec3, now: f64) {
        self.history.push_back(VelocitySample {
            sampled_at: now,
            velocity,
        });
        while self.is_oldest_stale(now) {
            self.history.pop_front();
        }
        let Some(anchor) = self.history.front() else {
            return;
        };
        let seconds = now - anchor.sampled_at;
        let has_baseline = seconds >= ACCELERATION_WINDOW.as_secs_f64() / 2.0;
        if !has_baseline {
            return;
        }
        self.linear = scale(subtract(velocity, anchor.velocity), 1.0 / seconds);
    }
}
