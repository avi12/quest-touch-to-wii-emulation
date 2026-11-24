//! Accumulates the rotation steps of a twist and reports its axis.

use std::fmt;

use bridge::math::norm;
use geometry::{Mat3, Vec3};

use crate::twist::{dominant_eigen, relative_rotation};
use numeric::format::{rounded_vector, to_fixed};

/// Steps smaller than this (rad) are sensor noise.
const MIN_ROTATION: f64 = 1e-4;
const AXIS_DECIMALS: usize = 3;
const PURITY_DECIMALS: usize = 2;

/// Sum of outer products of every rotation step since the first pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwistRecorder {
    covariance: Mat3,
    samples: usize,
    previous: Mat3,
}

impl TwistRecorder {
    /// Starts from the controller's current rotation.
    pub fn new(start: Mat3) -> Self {
        Self {
            covariance: [[0.0; 3]; 3],
            samples: 0,
            previous: start,
        }
    }

    /// Adds the step to `current`; repeated poses (no new tracking data) are skipped.
    pub fn observe(&mut self, current: Mat3) {
        let is_repeat = current == self.previous;
        if is_repeat {
            return;
        }
        let step = relative_rotation(self.previous, current);
        let is_motion = norm(step) > MIN_ROTATION;
        if is_motion {
            self.add_outer_product(step);
            self.samples += 1;
        }
        self.previous = current;
    }

    fn add_outer_product(&mut self, vector: Vec3) {
        for (row, row_factor) in self.covariance.iter_mut().zip(vector) {
            for (cell, column_factor) in row.iter_mut().zip(vector) {
                *cell += row_factor * column_factor;
            }
        }
    }

    /// The twist axis, or `None` when no rotation was recorded.
    pub fn axis(&self) -> Option<TwistAxis> {
        let [[x_x, _, _], [_, y_y, _], [_, _, z_z]] = self.covariance;
        let trace = x_x + y_y + z_z;
        let is_still = self.samples == 0 || trace == 0.0;
        if is_still {
            return None;
        }
        let eigen = dominant_eigen(self.covariance);
        Some(TwistAxis {
            samples: self.samples,
            axis: eigen.unit,
            purity: eigen.eigenvalue / trace,
        })
    }
}

/// The measured twist axis in the controller frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwistAxis {
    pub samples: usize,
    /// Unit axis; its sign is arbitrary.
    pub axis: Vec3,
    /// Share of the motion around `axis`: 1.0 is a perfectly clean twist.
    pub purity: f64,
}

impl fmt::Display for TwistAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} samples | axis {} (sign is arbitrary) | purity {} (1.0 = a perfectly clean twist)",
            self.samples,
            rounded_vector(&self.axis, AXIS_DECIMALS),
            to_fixed(self.purity, PURITY_DECIMALS),
        )
    }
}

#[cfg(test)]
mod tests {
    use geometry::Mat3;

    use crate::twist::TwistRecorder;

    const STEPS: u32 = 20;
    const STEP_ANGLE: f64 = 0.01;

    fn roll(angle: f64) -> Mat3 {
        [
            [angle.cos(), -angle.sin(), 0.0],
            [angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn a_pure_roll_gives_the_z_axis() {
        let mut recorder = TwistRecorder::new(roll(0.0));
        assert!(recorder.axis().is_none());
        for i_step in 1..=STEPS {
            let rotation = roll(f64::from(i_step) * STEP_ANGLE);
            recorder.observe(rotation);
            recorder.observe(rotation);
        }
        let twist = recorder.axis().unwrap();
        assert_eq!(twist.samples, 20);
        assert!((twist.axis[2].abs() - 1.0).abs() < 1e-9);
        assert!(twist.to_string().contains("purity 1.00"));
    }
}
