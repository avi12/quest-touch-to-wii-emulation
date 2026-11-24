//! The fields of a DSU pad packet that the probe reports.

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
    /// Right stick X (the pointer), 128 = centre.
    pub pointer_x: u8,
    /// Right stick Y (the pointer), 128 = centre.
    pub pointer_y: u8,
    /// Both button bytes, the second one in the high byte.
    pub buttons: u16,
    /// The largest absolute gyro rate of the three axes (deg/s).
    pub peak_gyro: f64,
}

impl PadReading {
    /// Decodes a pad packet, or `None` when the datagram is too short to be one.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < PAD_PACKET_MIN_BYTES {
            return None;
        }
        let byte = |offset: usize| data.get(offset).copied().unwrap_or(0);
        let peak_gyro = (0..GYRO_AXES)
            .filter_map(|axis| float_at(data, GYRO + axis * FLOAT_BYTES))
            .map(|rate| f64::from(rate).abs())
            .fold(0.0, f64::max);
        Some(Self {
            pointer_x: byte(POINTER_X),
            pointer_y: byte(POINTER_Y),
            buttons: u16::from_le_bytes([byte(BUTTONS_1), byte(BUTTONS_2)]),
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
    use super::*;

    #[test]
    fn decodes_the_reported_fields() {
        let mut data = vec![0; PAD_PACKET_MIN_BYTES];
        data[BUTTONS_1] = 0x12;
        data[BUTTONS_2] = 0x34;
        data[POINTER_X] = 200;
        data[POINTER_Y] = 7;
        data[GYRO + FLOAT_BYTES..GYRO + 2 * FLOAT_BYTES]
            .copy_from_slice(&(-90.5_f32).to_le_bytes());
        let reading = PadReading::parse(&data).unwrap();
        assert_eq!(reading.buttons, 0x3412);
        assert_eq!((reading.pointer_x, reading.pointer_y), (200, 7));
        assert!((reading.peak_gyro - 90.5).abs() < f64::EPSILON);
        assert!(PadReading::parse(&data[..PAD_PACKET_MIN_BYTES - 1]).is_none());
    }
}
