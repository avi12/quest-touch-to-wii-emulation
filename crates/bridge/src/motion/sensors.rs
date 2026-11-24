use crate::math::{
    DegreesPerSecond, Gravities, Mat3, Radians, Vec3, WORLD_UP, add, multiply_matrix,
    multiply_transposed, scale,
};
use crate::motion::swing::measure_swing;
use crate::motion::{Motion, Sensors};

/// Standard gravity, m/s^2.
const GRAVITY: f64 = 9.806_65;
const FLIP_ACCEL: Vec3 = [1.0, 1.0, 1.0]; // tweak if Dolphin shows an axis inverted
const FLIP_GYRO: Vec3 = [1.0, 1.0, 1.0];
const ROLL_FADE_LOW: f64 = 0.25; // roll about the aim axis is meaningless when the aim is near vertical: gravity then has no sideways part
const ROLL_FADE_HIGH: f64 = 0.5; // sideways gravity (0..1) at and above which roll is passed in full
const ROLL_GAIN: f64 = 1.0; // 1 = hand follows the controller's roll 1:1; negative flips the direction

/// Controller frame -> Wiimote frame (right, forward, up) using the pointing basis.
fn to_wiimote_frame(basis: Mat3, vector: Vec3) -> Vec3 {
    multiply_matrix(basis, vector)
}

/// How much of the roll to pass on (0..1), from how much of gravity lies across the aim axis.
fn roll_weight(gravity: Vec3) -> f64 {
    let sideways = gravity[0].hypot(gravity[2]);
    ((sideways - ROLL_FADE_LOW) / (ROLL_FADE_HIGH - ROLL_FADE_LOW)).clamp(0.0, 1.0)
}

/// Wiimote-frame motion in Dolphin's DSU convention (read from Dolphin's DualShockUDPClient.cpp):
/// accel (g): x = -right, y = -up, z = forward; gyro (deg/s): roll = +about forward.
/// Dolphin's motion-sensor pointer turns the hand by the Wiimote's roll, so it is fed ROLL ONLY
/// (aim comes from the pointer): gravity is replaced by a version rotated by the roll about the aim
/// axis, yaw/pitch rates are dropped, swings stay.
#[must_use]
pub fn to_sensors(rotation: Mat3, basis: Mat3, motion: Motion) -> Sensors {
    let [accel_x, accel_y, accel_z] = motion.acceleration.0;
    let specific_force = scale(
        multiply_transposed(rotation, [accel_x, accel_y + GRAVITY, accel_z]),
        1.0 / GRAVITY,
    );
    let angular_in_wiimote =
        to_wiimote_frame(basis, multiply_transposed(rotation, motion.angular_rate.0));
    let roll_rate = Radians(angular_in_wiimote[1]).to_degrees().0;
    let gravity_in_wiimote = to_wiimote_frame(basis, multiply_transposed(rotation, WORLD_UP));
    let weight = roll_weight(gravity_in_wiimote);
    let roll = gravity_in_wiimote[0].atan2(gravity_in_wiimote[2]) * ROLL_GAIN * weight;
    let roll_only_gravity = [roll.sin(), 0.0, roll.cos()];
    let swing = measure_swing(to_wiimote_frame(basis, specific_force), gravity_in_wiimote);
    let [right, forward, upward] = add(roll_only_gravity, swing.acceleration);
    Sensors {
        accel: Gravities([
            -right * FLIP_ACCEL[0],
            -upward * FLIP_ACCEL[1],
            forward * FLIP_ACCEL[2],
        ]),
        // no roll rate during a swing: Dolphin would integrate it into a lasting tilt
        gyro: DegreesPerSecond([
            0.0,
            0.0,
            roll_rate * weight * (1.0 - swing.factor) * FLIP_GYRO[2],
        ]),
    }
}
