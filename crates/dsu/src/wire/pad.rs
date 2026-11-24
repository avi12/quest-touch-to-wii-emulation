//! One pad snapshot and its DSU pad-data body.

use num_traits::ToPrimitive;

use crate::wire::buttons::{Buttons1, Buttons2};
use crate::wire::protocol::SLOT_INFO;

const ANALOG_PRESSED: u8 = 255;
const STICK_MAX: f64 = 255.0;
const STICK_HALF_RANGE: f64 = 127.5;
const TOUCHPAD_BYTES: usize = 12;
/// Slot info, head, 20 button / stick / analog bytes, touchpads, timestamp, six floats.
const PAD_BODY_BYTES: usize = SLOT_INFO.len() + 5 + 20 + TOUCHPAD_BYTES + 8 + 6 * 4;

/// One pad snapshot: digital bits, stick / trigger bytes, microsecond timestamp, accel in g,
/// gyro in deg/s.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadState {
    /// Share, L3, R3, Options and the D-pad.
    pub buttons1: Buttons1,
    /// Shoulders, triggers and face buttons.
    pub buttons2: Buttons2,
    /// `[LX, LY, RX, RY]`, 128 = centre, Y 255 = up.
    pub sticks: [u8; 4],
    /// `[R1, L1, R2, L2]` analog bytes.
    pub triggers: [u8; 4],
    /// Motion timestamp in microseconds.
    pub ts_us: u64,
    /// Acceleration in g, DSU axes: x = -right, y = -up, z = forward.
    pub accel: [f64; 3],
    /// Angular rate in deg/s: (pitch, yaw, roll).
    pub gyro: [f64; 3],
}

/// Stick value -1..1 -> 0..255 (128 = centre); NaN maps to 0 as the TypeScript bridge's buffer
/// write did.
pub fn stick_byte(value: f64) -> u8 {
    ((value + 1.0) * STICK_HALF_RANGE)
        .round()
        .clamp(0.0, STICK_MAX)
        .to_u8()
        .unwrap_or(0)
}

fn analog(pressed: bool) -> u8 {
    if pressed { ANALOG_PRESSED } else { 0 }
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
    let mut body = Vec::with_capacity(PAD_BODY_BYTES);
    body.extend_from_slice(&SLOT_INFO);
    body.push(1); // connected
    body.extend_from_slice(&packet_number.to_le_bytes());
    body.extend_from_slice(&[bits1.0, bits2.0, u8::from(is_recentering), 0]);
    body.extend_from_slice(&pad.sticks);
    body.extend_from_slice(&dpad);
    body.extend_from_slice(&face);
    body.extend_from_slice(&pad.triggers);
    body.extend_from_slice(&[0; TOUCHPAD_BYTES]);
    body.extend_from_slice(&pad.ts_us.to_le_bytes());
    for value in pad.accel.iter().chain(&pad.gyro) {
        body.extend_from_slice(&single(*value).to_le_bytes());
    }
    body
}

/// f64 -> f32 with the IEEE rounding of a JS `writeFloatLE` (overflow becomes infinity).
fn single(value: f64) -> f32 {
    value.to_f32().unwrap_or(f32::NAN)
}

/// A pad exercising every field, matching the golden TypeScript captures.
#[cfg(test)]
pub(crate) fn sample_pad() -> PadState {
    PadState {
        buttons1: Buttons1::DPAD_UP | Buttons1::SHARE,
        buttons2: Buttons2::FACE_A | Buttons2::FACE_Y | Buttons2::SHOULDER_R1,
        sticks: [128, 255, 0, 64],
        triggers: [255, 0, 10, 20],
        ts_us: 1_234_567_890_123,
        accel: [0.1, -1.0, 0.25],
        gyro: [12.5, -3.3, 1e40],
    }
}

#[cfg(test)]
mod tests {
    use super::{PAD_BODY_BYTES, PadState, pad_body, stick_byte};
    #[test]
    fn stick_bytes_match_typescript() {
        let values = [-2.0, -1.0, -0.5, 0.0, 0.001, 0.5, 1.0, 2.0, f64::NAN];
        assert_eq!(
            values.map(stick_byte),
            [0, 0, 64, 128, 128, 191, 255, 255, 0]
        );
    }

    #[test]
    fn body_has_fixed_length() {
        assert_eq!(
            pad_body(&PadState::default(), 0, false).len(),
            PAD_BODY_BYTES
        );
    }
}
