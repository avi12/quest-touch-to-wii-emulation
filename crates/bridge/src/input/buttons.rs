//! Touch controllers -> one DSU pad, bound as `config/` expects in Dolphin.

use std::ops::BitOrAssign;
use std::time::Duration;

use dsu::{Buttons1, Buttons2, PadState, Shoulders, Stick};
use openvr_sys::Button;

use crate::input::Controllers;
use crate::motion::Sensors;
use crate::pointer::PointerPosition;

/// An analog trigger or grip counts as a press above this.
const PRESS_THRESHOLD: f64 = 0.5;
/// Right stick -> D-pad (the stick axes carry the pointer).
const STICK_THRESHOLD: f64 = 0.6;
const BYTE_MAX: f64 = 255.0;
const BYTE_VALUES: i64 = 256;

/// Everything one pad snapshot is built from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PadInput {
    pub controllers: Controllers,
    /// The left controller is switched on and tracked: the Nunchuk is plugged in.
    pub is_left_attached: bool,
    pub pointer: PointerPosition,
    pub sensors: Sensors,
    /// Wall clock since the Unix epoch, in whole milliseconds.
    pub wall_clock: Duration,
}

fn combine<T: Copy + Default + BitOrAssign>(rules: &[(bool, T)]) -> T {
    let mut flags = T::default();
    for (is_on, flag) in rules {
        if *is_on {
            flags |= *flag;
        }
    }
    flags
}

/// `Math.trunc(analog * 255)` stored as a byte the way the TypeScript `Buffer.from` did (mod 256).
fn to_byte(analog: f64) -> u8 {
    let truncated = numeric::truncate_to_i64(analog * BYTE_MAX).unwrap_or(0);
    u8::try_from(truncated.rem_euclid(BYTE_VALUES)).unwrap_or(0)
}

fn first_buttons(Controllers { right, left }: &Controllers, is_left_attached: bool) -> Buttons1 {
    let [stick_x, stick_y] = right.stick;
    combine(&[
        (stick_y > STICK_THRESHOLD, Buttons1::DPAD_UP),
        (stick_y < -STICK_THRESHOLD, Buttons1::DPAD_DOWN),
        (stick_x > STICK_THRESHOLD, Buttons1::DPAD_RIGHT),
        (stick_x < -STICK_THRESHOLD, Buttons1::DPAD_LEFT),
        (right.is_down(Button::Click), Buttons1::STICK_R3),
        // Dolphin's Extension is bound to L3: on = Nunchuk plugged in.
        (is_left_attached, Buttons1::STICK_L3),
        (left.is_down(Button::System), Buttons1::OPTIONS),
        (right.is_down(Button::System), Buttons1::SHARE),
    ])
}

fn second_buttons(Controllers { right, left }: &Controllers) -> Buttons2 {
    combine(&[
        (right.is_down(Button::Primary), Buttons2::FACE_A),
        (right.is_down(Button::Secondary), Buttons2::FACE_B),
        (left.is_down(Button::Primary), Buttons2::FACE_X),
        // Dolphin's Wii button 2 is bound to the analog R2 (see `shoulders`).
        (left.is_down(Button::Secondary), Buttons2::TRIGGER_R2),
        // Dolphin's Wii Home is bound to Triangle.
        (right.trig > PRESS_THRESHOLD, Buttons2::FACE_Y),
        (left.trig > PRESS_THRESHOLD, Buttons2::TRIGGER_L2),
        (right.grip > PRESS_THRESHOLD, Buttons2::SHOULDER_R1),
        (left.grip > PRESS_THRESHOLD, Buttons2::SHOULDER_L1),
    ])
}

fn shoulders(Controllers { right, left }: &Controllers) -> Shoulders {
    let is_left_b = left.is_down(Button::Secondary);
    Shoulders {
        right_shoulder: to_byte(right.grip),
        left_shoulder: to_byte(left.grip),
        right_trigger: to_byte(f64::from(u8::from(is_left_b))),
        left_trigger: to_byte(left.trig),
    }
}

/// One DSU pad snapshot from both controllers, the pointer and the motion sensors.
pub fn build_pad(input: &PadInput) -> PadState {
    let PadInput {
        controllers,
        pointer,
        sensors,
        ..
    } = input;
    let [left_x, left_y] = controllers.left.stick;
    PadState {
        buttons1: first_buttons(controllers, input.is_left_attached),
        buttons2: second_buttons(controllers),
        left_stick: Stick::from_unit(left_x, left_y),
        right_stick: Stick::from_unit(pointer.x, pointer.y),
        shoulders: shoulders(controllers),
        timestamp: input.wall_clock,
        accel: sensors.accel.0,
        gyro: sensors.gyro.0,
    }
}

#[cfg(test)]
mod tests;
