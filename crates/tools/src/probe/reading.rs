//! The fields of a DSU pad packet that the probe reports.

use dsu::{Buttons1, Buttons2, Stick};

const PAD_PACKET_MIN_BYTES: usize = 100;
const BUTTONS_1: usize = 36;
const BUTTONS_2: usize = 37;
const POINTER_X: usize = 42;
const POINTER_Y: usize = 43;
const GYRO: usize = 88;
const GYRO_AXES: usize = 3;
const FLOAT_BYTES: usize = 4;

/// One pad packet as the probe sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PadReading {
    /// The right stick (the pointer), 128 = centre.
    pub pointer: Stick,
    pub buttons1: Buttons1,
    pub buttons2: Buttons2,
    /// The largest absolute gyro rate of the three axes (deg/s).
    pub peak_gyro: f64,
}

impl PadReading {
    /// Decodes a pad packet, or `None` when the datagram is too short to be one.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let is_too_short = data.len() < PAD_PACKET_MIN_BYTES;
        if is_too_short {
            return None;
        }
        let byte = |offset: usize| data.get(offset).copied().unwrap_or(0);
        let peak_gyro = (0..GYRO_AXES)
            .filter_map(|axis| float_at(data, GYRO + axis * FLOAT_BYTES))
            .map(|rate| f64::from(rate).abs())
            .fold(0.0, f64::max);
        Some(Self {
            pointer: Stick {
                x: byte(POINTER_X),
                y: byte(POINTER_Y),
            },
            buttons1: Buttons1(byte(BUTTONS_1)),
            buttons2: Buttons2(byte(BUTTONS_2)),
            peak_gyro,
        })
    }
}

fn float_at(data: &[u8], offset: usize) -> Option<f32> {
    let bytes = data.get(offset..offset + FLOAT_BYTES)?;
    bytes.try_into().ok().map(f32::from_le_bytes)
}

#[cfg(test)]
mod tests {
    use dsu::{Buttons1, Buttons2, Stick};

    use crate::probe::PadReading;
    use crate::probe::reading::{
        BUTTONS_1, BUTTONS_2, FLOAT_BYTES, GYRO, PAD_PACKET_MIN_BYTES, POINTER_X, POINTER_Y,
    };

    const BUTTON_BYTES: (u8, u8) = (0x12, 0x34);
    const POINTER: Stick = Stick { x: 200, y: 7 };
    const GYRO_Y: f32 = -90.5;

    #[test]
    fn decodes_the_reported_fields() {
        let mut data = vec![0; PAD_PACKET_MIN_BYTES];
        let (first_byte, second_byte) = BUTTON_BYTES;
        data[BUTTONS_1] = first_byte;
        data[BUTTONS_2] = second_byte;
        data[POINTER_X] = POINTER.x;
        data[POINTER_Y] = POINTER.y;
        data[GYRO + FLOAT_BYTES..GYRO + 2 * FLOAT_BYTES].copy_from_slice(&GYRO_Y.to_le_bytes());
        let reading = PadReading::parse(&data);
        assert_eq!(
            reading.map(|decoded| (decoded.buttons1, decoded.buttons2, decoded.pointer)),
            Some((Buttons1(first_byte), Buttons2(second_byte), POINTER))
        );
        assert!(reading.is_some_and(|decoded| (decoded.peak_gyro - 90.5).abs() < f64::EPSILON));
        assert!(PadReading::parse(&data[..PAD_PACKET_MIN_BYTES - 1]).is_none());
    }
}
