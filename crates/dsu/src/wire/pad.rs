//! One pad snapshot and its DSU pad-data body.

use std::time::Duration;

use geometry::Vec3;

use crate::wire::analog::{Shoulders, Stick};
use crate::wire::buttons::{Buttons1, Buttons2};
use crate::wire::float::narrow;
use crate::wire::protocol::SLOT_INFO;

/// Analog value of a held D-pad or face button.
const ANALOG_PRESSED: u8 = 255;
/// The connected flag after the slot info.
const CONNECTED: u8 = 1;
/// Two touchpad points, six bytes each, always untouched.
const TOUCHPAD_BYTES: usize = 12;
/// Bytes of one motion float.
const FLOAT_BYTES: usize = size_of::<f32>();
/// Slot info, connected flag, packet number, 4 button bytes, sticks, D-pad, face, shoulders,
/// touchpads, timestamp, six floats.
const PAD_BODY_BYTES: usize = SLOT_INFO.len()
    + 1
    + size_of::<u32>()
    + 4
    + 4
    + 4
    + 4
    + 4
    + TOUCHPAD_BYTES
    + size_of::<u64>()
    + 6 * FLOAT_BYTES;

/// One pad snapshot: digital bits, stick / shoulder bytes, motion timestamp, accel in g, gyro in
/// deg/s.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadState {
    /// Share, L3, R3, Options and the D-pad.
    pub buttons1: Buttons1,
    /// Shoulders, triggers and face buttons.
    pub buttons2: Buttons2,
    /// Left stick.
    pub left_stick: Stick,
    /// Right stick.
    pub right_stick: Stick,
    /// Analog shoulders and triggers.
    pub shoulders: Shoulders,
    /// Motion timestamp, sent in whole microseconds.
    pub timestamp: Duration,
    /// Acceleration in g, DSU axes: x = -right, y = -up, z = forward.
    pub accel: Vec3,
    /// Angular rate in deg/s: (pitch, yaw, roll).
    pub gyro: Vec3,
}

fn analog(is_pressed: bool) -> u8 {
    if is_pressed { ANALOG_PRESSED } else { 0 }
}

/// The pad-data body (after the event id) for one snapshot.
pub(crate) fn pad_body(pad: &PadState, packet_number: u32, is_recentering: bool) -> Vec<u8> {
    let (bits1, bits2) = (pad.buttons1, pad.buttons2);
    // Dolphin reads the D-pad and the face buttons from the ANALOG fields, so mirror the bits.
    let dpad = [
        Buttons1::DPAD_LEFT,
        Buttons1::DPAD_DOWN,
        Buttons1::DPAD_RIGHT,
        Buttons1::DPAD_UP,
    ]
    .map(|mask| analog(bits1.contains(mask)));
    // Dolphin's field order is Square, Cross, Circle, Triangle (not the spec's Y, B, A, X).
    let face = [
        Buttons2::FACE_X,
        Buttons2::FACE_A,
        Buttons2::FACE_B,
        Buttons2::FACE_Y,
    ]
    .map(|mask| analog(bits2.contains(mask)));
    let microseconds = u64::try_from(pad.timestamp.as_micros()).unwrap_or(u64::MAX);
    let mut body = Vec::with_capacity(PAD_BODY_BYTES);
    body.extend_from_slice(&SLOT_INFO);
    body.push(CONNECTED);
    body.extend_from_slice(&packet_number.to_le_bytes());
    body.extend_from_slice(&[bits1.0, bits2.0, u8::from(is_recentering), 0]);
    body.extend_from_slice(&pad.left_stick.bytes());
    body.extend_from_slice(&pad.right_stick.bytes());
    body.extend_from_slice(&dpad);
    body.extend_from_slice(&face);
    body.extend_from_slice(&pad.shoulders.bytes());
    body.extend_from_slice(&[0; TOUCHPAD_BYTES]);
    body.extend_from_slice(&microseconds.to_le_bytes());
    for value in pad.accel.iter().chain(&pad.gyro) {
        body.extend_from_slice(&narrow(*value).to_le_bytes());
    }
    body
}

/// A pad exercising every field, matching the golden TypeScript captures.
#[cfg(test)]
pub(crate) const SAMPLE_PAD: PadState = PadState {
    buttons1: Buttons1(Buttons1::DPAD_UP.0 | Buttons1::SHARE.0),
    buttons2: Buttons2(Buttons2::FACE_A.0 | Buttons2::FACE_Y.0 | Buttons2::SHOULDER_R1.0),
    left_stick: Stick { x: 128, y: 255 },
    right_stick: Stick { x: 0, y: 64 },
    shoulders: Shoulders {
        right_shoulder: 255,
        left_shoulder: 0,
        right_trigger: 10,
        left_trigger: 20,
    },
    timestamp: Duration::from_micros(1_234_567_890_123),
    accel: [0.1, -1.0, 0.25],
    gyro: [12.5, -3.3, 1e40],
};

#[cfg(test)]
mod tests {
    use crate::wire::pad::{PAD_BODY_BYTES, PadState, SAMPLE_PAD, pad_body};

    #[test]
    fn body_has_fixed_length() {
        assert_eq!(
            pad_body(&PadState::default(), 0, false).len(),
            PAD_BODY_BYTES
        );
        assert_eq!(pad_body(&SAMPLE_PAD, 0, true).len(), PAD_BODY_BYTES);
    }
}
