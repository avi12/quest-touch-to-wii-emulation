//! Golden values from running src/motion.ts on the same inputs.

use std::time::Duration;

use crate::math::{
    Mat3, MetresPerSecond, MetresPerSecondSquared, RadiansPerSecond, Vec3, rotation_from_angles,
};
use crate::motion::{Motion, MotionTracker, PoseSample, to_sensors};
use crate::pointer::{Pointer, PointerInput};

const TOLERANCE: f64 = 1e-9;
const FRAME_SECONDS: f64 = 0.004;
const FRAME: Duration = Duration::from_millis(4);
const SAMPLES: u8 = 10;
/// Yaw, pitch, roll the TS pointer recentred at.
const RECENTER_ANGLES: Vec3 = [0.12, 0.05, 0.3];
/// Yaw, pitch, roll of the controller the sensors are read at.
const SENSED_ANGLES: Vec3 = [0.3, 0.2, 0.4];
/// Per-sample velocity factors: x grows with step squared, y with step, z with sin(step).
const VELOCITY_FACTORS: Vec3 = [0.5, -2.0, 0.3];
/// Per-sample angular velocity: x and z grow with the step, y stays.
const ANGULAR_FACTORS: Vec3 = [0.1, -0.2, 0.05];
const GOLDEN_ACCELERATION: Vec3 = [
    6.462_900_545_664_061,
    -1.999_549_624_921_875_2,
    23.947_834_456_765_513,
];
const GOLDEN_ANGULAR_RATE: Vec3 = [0.9, -0.2, 0.45];
const GOLDEN_SWING_ACCEL: Vec3 = [
    -0.692_451_124_347_558_1,
    0.545_078_580_474_387_7,
    -2.848_507_602_901_338,
];
const GENTLE_MOTION: Motion = Motion {
    acceleration: MetresPerSecondSquared([0.5, 0.2, -0.3]),
    angular_rate: RadiansPerSecond([1.0, 2.0, 3.0]),
};
const GOLDEN_GENTLE_ACCEL: Vec3 = [-0.069_169_110_687_112_8, -0.997_604_948_928_559_6, 0.0];
const GOLDEN_GENTLE_GYRO: Vec3 = [0.0, 0.0, -204.976_048_021_184_82];

fn assert_close(actual: Vec3, expected: Vec3) {
    for (value, wanted) in actual.iter().zip(expected) {
        assert!(
            (value - wanted).abs() < TOLERANCE,
            "{actual:?} vs {expected:?}"
        );
    }
}

fn rotation(angles: Vec3) -> Mat3 {
    let [yaw, pitch, roll] = angles;
    rotation_from_angles(yaw, pitch, roll)
}

/// The basis the TS pointer settled on after recentering at this rotation.
fn recentered_basis() -> Mat3 {
    let mut pointer = Pointer::new();
    pointer.update(&PointerInput {
        rotation: rotation(RECENTER_ANGLES),
        is_click_pressed: false,
        delta: FRAME,
        now: Duration::ZERO,
    });
    pointer.basis()
}

fn tracked_motion() -> Motion {
    let mut tracker = MotionTracker::new();
    let mut motion = Motion::default();
    let [velocity_x, velocity_y, velocity_z] = VELOCITY_FACTORS;
    let [angular_x, angular_y, angular_z] = ANGULAR_FACTORS;
    for i in 0..SAMPLES {
        let step = f64::from(i);
        motion = tracker.update(&PoseSample {
            position: [0.0; 3],
            rotation: rotation([0.0; 3]),
            velocity: MetresPerSecond([
                velocity_x * step * step * FRAME_SECONDS,
                velocity_y * step * FRAME_SECONDS,
                velocity_z * step.sin(),
            ]),
            angular_velocity: RadiansPerSecond([angular_x * step, angular_y, angular_z * step]),
            timestamp: Duration::from_secs_f64(step * FRAME_SECONDS),
        });
    }
    motion
}

#[test]
fn tracker_matches_typescript() {
    let motion = tracked_motion();
    assert_close(motion.acceleration.0, GOLDEN_ACCELERATION);
    assert_close(motion.angular_rate.0, GOLDEN_ANGULAR_RATE);
}

#[test]
fn swing_drops_the_roll_rate() {
    let sensors = to_sensors(
        rotation(SENSED_ANGLES),
        recentered_basis(),
        tracked_motion(),
    );
    assert_close(sensors.accel.0, GOLDEN_SWING_ACCEL);
    assert_close(sensors.gyro.0, [0.0; 3]);
}

#[test]
fn gentle_motion_passes_roll_only() {
    let sensors = to_sensors(rotation(SENSED_ANGLES), recentered_basis(), GENTLE_MOTION);
    assert_close(sensors.accel.0, GOLDEN_GENTLE_ACCEL);
    assert_close(sensors.gyro.0, GOLDEN_GENTLE_GYRO);
}
