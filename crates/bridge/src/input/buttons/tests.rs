//! Golden values printed by the TypeScript `buildPad` for the same input.

use std::time::Duration;

use dsu::{Buttons1, Buttons2, PadState, Shoulders, Stick};
use openvr_sys::Button;

use crate::input::{Controllers, Controls, PadInput, build_pad};
use crate::math::{DegreesPerSecond, Gravities};
use crate::motion::{LEVEL_STILL_SENSORS, Sensors};
use crate::pointer::PointerPosition;

/// Stick, trigger, grip of the busy controller.
const BUSY: ([f64; 2], f64, f64) = ([0.7, -0.65], 0.6, 0.999);
/// Stick, trigger, grip of the calmer controller.
const CALM: ([f64; 2], f64, f64) = ([-0.3, 0.42], 0.51, 0.2);
const ATTACHED_SENSORS: Sensors = Sensors {
    accel: Gravities([0.1, -1.0, 0.2]),
    gyro: DegreesPerSecond([1.0, 2.0, 3.0]),
};
const ATTACHED_POINTER: PointerPosition = PointerPosition { x: 0.25, y: -0.8 };
const PARKED_POINTER: PointerPosition = PointerPosition { x: -1.0, y: 0.0 };
const WALL_CLOCK: Duration = Duration::from_millis(1_700_000_000_123);

fn controls((stick, trig, grip): ([f64; 2], f64, f64), held: &[Button]) -> Controls {
    let mut controls = Controls::default().holding(held);
    controls.stick = stick;
    controls.trig = trig;
    controls.grip = grip;
    controls
}

fn first_controls() -> Controls {
    controls(BUSY, &[Button::Primary, Button::Click, Button::System])
}

fn second_controls() -> Controls {
    controls(CALM, &[Button::Secondary])
}

fn pad(right: Controls, left: Controls, is_left_attached: bool) -> PadState {
    let (pointer, sensors) = if is_left_attached {
        (ATTACHED_POINTER, ATTACHED_SENSORS)
    } else {
        (PARKED_POINTER, LEVEL_STILL_SENSORS)
    };
    build_pad(&PadInput {
        controllers: Controllers { right, left },
        is_left_attached,
        pointer,
        sensors,
        wall_clock: WALL_CLOCK,
    })
}

#[test]
fn matches_typescript() {
    let first = pad(first_controls(), second_controls(), true);
    assert_eq!((first.buttons1.0, first.buttons2.0), (103, 171));
    assert_eq!(
        (first.left_stick, first.right_stick),
        (Stick { x: 89, y: 181 }, Stick { x: 159, y: 25 })
    );
    assert_eq!(
        first.shoulders,
        Shoulders {
            right_shoulder: 254,
            left_shoulder: 51,
            right_trigger: 255,
            left_trigger: 130,
        }
    );
    assert_eq!(first.timestamp.as_micros(), 1_700_000_000_123_000);
    assert_eq!(first.gyro, [1.0, 2.0, 3.0]);
    let second = pad(second_controls(), first_controls(), false);
    assert_eq!((second.buttons1.0, second.buttons2.0), (8, 213));
    assert_eq!(
        (second.left_stick, second.right_stick),
        (Stick { x: 217, y: 45 }, Stick { x: 0, y: 128 })
    );
    assert_eq!(
        second.shoulders,
        Shoulders {
            right_shoulder: 51,
            left_shoulder: 254,
            right_trigger: 0,
            left_trigger: 153,
        }
    );
}

#[test]
fn nothing_held_is_an_empty_pad() {
    let idle = pad(Controls::default(), Controls::default(), false);
    assert_eq!(
        (idle.buttons1, idle.buttons2),
        (Buttons1::NONE, Buttons2::NONE)
    );
    assert_eq!(idle.shoulders, Shoulders::default());
}
