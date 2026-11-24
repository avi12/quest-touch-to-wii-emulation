//! Optional live stream of what the bridge sees and sends, as JSON lines, so the motion can be
//! inspected while it happens. `tools/telemetry_tail` reads these field names.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use num_traits::ToPrimitive;
use serde_json::Value;

use crate::math::{Mat3, Vec3, norm, radians_to_degrees, round_hundredths};
use crate::motion::Sensors;
use crate::pointer::PointerPosition;

/// 50 samples per second.
const SAMPLE_EVERY_MS: u64 = 20;

/// One frame as the telemetry records it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TelemetrySample {
    pub direction: Vec3,
    pub pointer: PointerPosition,
    pub sensors: Sensors,
    pub did_recenter: bool,
    pub rotation: Mat3,
    pub facing: f64,
    /// B held: the person is recording a take to analyse.
    pub is_marked: bool,
}

/// [`round_hundredths`] as JSON the way `JSON.stringify` writes it: whole numbers without a
/// fraction, `-0` as `0`, non-finite as `null`.
fn round(value: f64) -> Value {
    let rounded = round_hundredths(value);
    match rounded.to_i64() {
        Some(whole) if rounded.fract() == 0.0 => Value::from(whole),
        _ => Value::from(rounded),
    }
}

/// The JSON line (without the newline) written for one sample at `now_epoch_ms`, keys in the
/// TypeScript bridge's order.
pub fn telemetry_line(sample: &TelemetrySample, now_epoch_ms: u64) -> String {
    let [east, height, south] = sample.direction;
    let pitch = radians_to_degrees((height / norm(sample.direction)).asin());
    let yaw = radians_to_degrees(east.atan2(-south));
    let [_, _, gyro_roll] = sample.sensors.gyro;
    let rotation: Vec<Value> = sample
        .rotation
        .as_flattened()
        .iter()
        .copied()
        .map(round)
        .collect();
    let fields = [
        ("timestamp", Value::from(now_epoch_ms)),
        ("pitch", round(pitch)),
        ("yaw", round(yaw)),
        ("pointerX", round(sample.pointer.x)),
        ("pointerY", round(sample.pointer.y)),
        (
            "accel",
            Value::from(sample.sensors.accel.map(round).to_vec()),
        ),
        ("gyroRoll", round(gyro_roll)),
        ("rotation", Value::from(rotation)),
        ("facing", round(sample.facing)),
        ("recenter", Value::from(sample.did_recenter)),
        ("mark", Value::from(sample.is_marked)),
    ];
    let members: Vec<String> = fields
        .iter()
        .map(|(key, value)| format!("{}:{value}", Value::from(*key)))
        .collect();
    format!("{{{}}}", members.join(","))
}

/// Appends one line per sample to a file; does nothing when no path is given.
#[derive(Debug)]
pub struct Telemetry {
    file: Option<File>,
    last_written_ms: u64,
}

impl Telemetry {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        let file = path
            .map(|path| OpenOptions::new().create(true).append(true).open(path))
            .transpose()?;
        Ok(Self {
            file,
            last_written_ms: 0,
        })
    }

    /// Writes the sample when 20 ms have passed since the last one, or on a recenter.
    pub fn record(&mut self, sample: &TelemetrySample, now_epoch_ms: u64) -> io::Result<()> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };
        let is_due = now_epoch_ms.saturating_sub(self.last_written_ms) >= SAMPLE_EVERY_MS;
        if !is_due && !sample.did_recenter {
            return Ok(());
        }
        self.last_written_ms = now_epoch_ms;
        let line = telemetry_line(sample, now_epoch_ms);
        file.write_all(format!("{line}\n").as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_1_SQRT_2;

    use super::*;

    /// Golden line written by the TypeScript `Telemetry` for the same sample.
    #[test]
    fn matches_typescript() {
        let sample = TelemetrySample {
            direction: [0.3, 0.2, -0.9],
            pointer: PointerPosition {
                x: 0.125,
                y: -0.335,
            },
            sensors: Sensors {
                accel: [-0.004, -1.0, 0.256],
                gyro: [1.0, 2.0, -12.345],
            },
            did_recenter: true,
            rotation: [
                [1.0, 0.0, 0.0],
                [0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2],
                [0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
            ],
            facing: 0.98765,
            is_marked: false,
        };
        assert_eq!(telemetry_line(&sample, 1234), GOLDEN);
    }

    const GOLDEN: &str = r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"facing":0.99,"recenter":true,"mark":false}"#;
}
