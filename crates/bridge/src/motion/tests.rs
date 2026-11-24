//! Golden values from running src/motion.ts on the same inputs.

use super::{Motion, MotionTracker, PoseSample, to_sensors};
use crate::math::{Vec3, rotation_from_angles};
use crate::pointer::{Pointer, PointerInput};

const TOLERANCE: f64 = 1e-9;
const FRAME_SECONDS: f64 = 0.004;
const SAMPLES: u8 = 10;

fn assert_close(actual: Vec3, expected: Vec3) {
    for (value, wanted) in actual.iter().zip(expected) {
        assert!(
            (value - wanted).abs() < TOLERANCE,
            "{actual:?} vs {expected:?}"
        );
    }
}

/// The basis the TS pointer settled on after recentering at this rotation.
fn recentered_basis() -> [Vec3; 3] {
    let mut pointer = Pointer::new();
    pointer.update(&PointerInput {
        rotation: rotation_from_angles(0.12, 0.05, 0.3),
        is_click_pressed: false,
        delta_seconds: FRAME_SECONDS,
        now_seconds: 0.0,
    });
    pointer.basis()
}

fn tracked_motion() -> Motion {
    let mut tracker = MotionTracker::new();
    let mut motion = Motion::default();
    for sample in 0..SAMPLES {
        let step = f64::from(sample);
        motion = tracker.update(&PoseSample {
            position: [0.0; 3],
            rotation: rotation_from_angles(0.0, 0.0, 0.0),
            velocity: [
                0.5 * step * step * FRAME_SECONDS,
                -2.0 * step * FRAME_SECONDS,
                0.3 * step.sin(),
            ],
            angular_velocity: [0.1 * step, -0.2, 0.05 * step],
            timestamp_seconds: step * FRAME_SECONDS,
        });
    }
    motion
}

#[test]
fn tracker_matches_typescript() {
    let motion = tracked_motion();
    assert_close(
        motion.acceleration,
        [
            6.462_900_545_664_061,
            -1.999_549_624_921_875_2,
            23.947_834_456_765_513,
        ],
    );
    assert_close(motion.angular_rate, [0.9, -0.2, 0.45]);
}

#[test]
fn swing_drops_the_roll_rate() {
    let sensors = to_sensors(
        rotation_from_angles(0.3, 0.2, 0.4),
        recentered_basis(),
        tracked_motion(),
    );
    assert_close(
        sensors.accel,
        [
            -0.692_451_124_347_558_1,
            0.545_078_580_474_387_7,
            -2.848_507_602_901_338,
        ],
    );
    assert_close(sensors.gyro, [0.0; 3]);
}

#[test]
fn gentle_motion_passes_roll_only() {
    let motion = Motion {
        acceleration: [0.5, 0.2, -0.3],
        angular_rate: [1.0, 2.0, 3.0],
    };
    let sensors = to_sensors(
        rotation_from_angles(0.3, 0.2, 0.4),
        recentered_basis(),
        motion,
    );
    assert_close(
        sensors.accel,
        [-0.069_169_110_687_112_8, -0.997_604_948_928_559_6, 0.0],
    );
    assert_close(sensors.gyro, [0.0, 0.0, -204.976_048_021_184_82]);
}
