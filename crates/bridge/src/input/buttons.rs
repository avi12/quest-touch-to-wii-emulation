//! Touch controllers -> one DSU pad, bound as `config/` expects in Dolphin.

use std::ops::BitOrAssign;

use dsu::{Buttons1, Buttons2, PadState, stick_byte};
use num_traits::ToPrimitive;
use openvr_sys::Button;

use super::Controllers;
use crate::motion::Sensors;
use crate::pointer::PointerPosition;

/// An analog trigger or grip counts as a press above this.
const PRESS_THRESHOLD: f64 = 0.5;
/// Right stick -> D-pad (the stick axes carry the pointer).
const STICK_THRESHOLD: f64 = 0.6;
const BYTE_MAX: f64 = 255.0;
const BYTE_VALUES: i64 = 256;
const MICROSECONDS_PER_MILLISECOND: u64 = 1000;

/// Everything one pad snapshot is built from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PadInput {
    pub controllers: Controllers,
    /// The left controller is switched on and tracked: the Nunchuk is plugged in.
    pub is_left_attached: bool,
    pub pointer: PointerPosition,
    pub sensors: Sensors,
    /// Wall clock, milliseconds since the Unix epoch.
    pub now_epoch_ms: u64,
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
    let truncated = (analog * BYTE_MAX).trunc().to_i64().unwrap_or(0);
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
        // Dolphin's Wii button 2 is bound to the analog R2 (see the triggers in build_pad).
        (left.is_down(Button::Secondary), Buttons2::TRIGGER_R2),
        // Dolphin's Wii Home is bound to Triangle.
        (right.trig > PRESS_THRESHOLD, Buttons2::FACE_Y),
        (left.trig > PRESS_THRESHOLD, Buttons2::TRIGGER_L2),
        (right.grip > PRESS_THRESHOLD, Buttons2::SHOULDER_R1),
        (left.grip > PRESS_THRESHOLD, Buttons2::SHOULDER_L1),
    ])
}

fn triggers(Controllers { right, left }: &Controllers) -> [u8; 4] {
    let is_left_b = left.is_down(Button::Secondary);
    [
        to_byte(right.grip),
        to_byte(left.grip),
        to_byte(f64::from(u8::from(is_left_b))),
        to_byte(left.trig),
    ]
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
        sticks: [left_x, left_y, pointer.x, pointer.y].map(stick_byte),
        triggers: triggers(controllers),
        ts_us: input.now_epoch_ms * MICROSECONDS_PER_MILLISECOND,
        accel: sensors.accel,
        gyro: sensors.gyro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Controls;

    fn controls(stick: [f64; 2], trig: f64, grip: f64, held: &[Button]) -> Controls {
        let mut controls = Controls::default().holding(held);
        controls.stick = stick;
        controls.trig = trig;
        controls.grip = grip;
        controls
    }

    fn first_controls() -> Controls {
        let held = [Button::Primary, Button::Click, Button::System];
        controls([0.7, -0.65], 0.6, 0.999, &held)
    }

    fn second_controls() -> Controls {
        controls([-0.3, 0.42], 0.51, 0.2, &[Button::Secondary])
    }

    fn pad(right: Controls, left: Controls, is_left_attached: bool) -> PadState {
        let (pointer, sensors) = if is_left_attached {
            let sensors = Sensors {
                accel: [0.1, -1.0, 0.2],
                gyro: [1.0, 2.0, 3.0],
            };
            (PointerPosition { x: 0.25, y: -0.8 }, sensors)
        } else {
            let still = crate::motion::LEVEL_STILL_SENSORS;
            (PointerPosition { x: -1.0, y: 0.0 }, still)
        };
        build_pad(&PadInput {
            controllers: Controllers { right, left },
            is_left_attached,
            pointer,
            sensors,
            now_epoch_ms: 1_700_000_000_123,
        })
    }

    /// Golden values printed by the TypeScript `buildPad` for the same input.
    #[test]
    fn matches_typescript() {
        let first = pad(first_controls(), second_controls(), true);
        assert_eq!((first.buttons1.0, first.buttons2.0), (103, 171));
        assert_eq!(first.sticks, [89, 181, 159, 25]);
        assert_eq!(first.triggers, [254, 51, 255, 130]);
        assert_eq!(first.ts_us, 1_700_000_000_123_000);
        assert_eq!(first.gyro, [1.0, 2.0, 3.0]);
        let second = pad(second_controls(), first_controls(), false);
        assert_eq!((second.buttons1.0, second.buttons2.0), (8, 213));
        assert_eq!(second.sticks, [217, 45, 0, 128]);
        assert_eq!(second.triggers, [51, 254, 0, 153]);
    }

    #[test]
    fn nothing_held_is_an_empty_pad() {
        let idle = pad(Controls::default(), Controls::default(), false);
        assert_eq!(
            (idle.buttons1, idle.buttons2),
            (Buttons1::NONE, Buttons2::NONE)
        );
        assert_eq!(idle.triggers, [0; 4]);
    }
}
