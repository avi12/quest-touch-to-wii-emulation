//! One telemetry line: the single definition of its fields, JSON keys and number format, shared by
//! the bridge that writes the lines and the tools that read them back.

use std::collections::HashMap;

use crate::math::{Mat3, Vec3, round_hundredths};
use crate::session::json::{JsonObject, JsonValue, parse_flat_object};

const TIMESTAMP_KEY: &str = "timestamp";
const PITCH_KEY: &str = "pitch";
const YAW_KEY: &str = "yaw";
const POINTER_X_KEY: &str = "pointerX";
const POINTER_Y_KEY: &str = "pointerY";
const ACCEL_KEY: &str = "accel";
const GYRO_ROLL_KEY: &str = "gyroRoll";
const ROTATION_KEY: &str = "rotation";
const FACING_KEY: &str = "facing";
const RECENTER_KEY: &str = "recenter";
const MARK_KEY: &str = "mark";

/// `facing` of a line written before it was recorded: aimed squarely at the screen.
const DEFAULT_FACING: f64 = 1.0;
const ROTATION_VALUES: usize = 9;
const ROW_LENGTH: usize = 3;

/// What the bridge saw and sent in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
    /// Wall clock, milliseconds since the Unix epoch.
    pub timestamp_ms: f64,
    /// Aim above the horizon, degrees.
    pub pitch: f64,
    /// Aim heading, degrees: 0 = straight ahead (-z), positive towards +x.
    pub yaw: f64,
    /// Pointer sent to Dolphin, -1..1 across the screen.
    pub pointer_x: f64,
    /// Pointer sent to Dolphin, -1..1 up the screen.
    pub pointer_y: f64,
    /// Accelerometer sent to Dolphin, g, DSU axes (x = -right, y = -up, z = forward).
    pub accel: Vec3,
    /// Gyro roll rate sent to Dolphin, degrees per second.
    pub gyro_roll: f64,
    /// Controller-to-world rotation, row-major.
    pub rotation: Mat3,
    /// Cosine between the aim and the screen's normal; low or negative is aimed away.
    pub facing: f64,
    /// The pointer recentred this frame.
    pub has_recentered: bool,
    /// B held: the person is recording a take to analyse.
    pub is_marked: bool,
}

/// A member as a number; `null` (how `JSON.stringify` writes NaN) reads as NaN.
fn as_number(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(number) => Some(*number),
        JsonValue::Null => Some(f64::NAN),
        _ => None,
    }
}

/// The members of one parsed line, read by type.
struct Members(HashMap<String, JsonValue>);

impl Members {
    fn number(&self, key: &str) -> Option<f64> {
        self.0.get(key).and_then(as_number)
    }

    fn boolean(&self, key: &str) -> Option<bool> {
        match self.0.get(key)? {
            JsonValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn array<const LENGTH: usize>(&self, key: &str) -> Option<[f64; LENGTH]> {
        match self.0.get(key)? {
            JsonValue::Numbers(values) => values.as_slice().try_into().ok(),
            _ => None,
        }
    }

    fn rotation(&self) -> Option<Mat3> {
        let flat: [f64; ROTATION_VALUES] = self.array(ROTATION_KEY)?;
        let (rows, _) = flat.as_chunks::<ROW_LENGTH>();
        rows.try_into().ok()
    }

    /// `facing`, or the default when the line predates it.
    fn facing(&self) -> Option<f64> {
        self.0
            .get(FACING_KEY)
            .map_or(Some(DEFAULT_FACING), as_number)
    }
}

impl Record {
    /// The JSON line (without the newline), keys in the TypeScript bridge's order and every value
    /// but the timestamp rounded to hundredths, numbers written as `JSON.stringify` writes them.
    pub fn to_json_line(&self) -> String {
        let rotation: Vec<f64> = self
            .rotation
            .as_flattened()
            .iter()
            .copied()
            .map(round_hundredths)
            .collect();
        let mut object = JsonObject::new();
        object.number(TIMESTAMP_KEY, self.timestamp_ms);
        object.number(PITCH_KEY, round_hundredths(self.pitch));
        object.number(YAW_KEY, round_hundredths(self.yaw));
        object.number(POINTER_X_KEY, round_hundredths(self.pointer_x));
        object.number(POINTER_Y_KEY, round_hundredths(self.pointer_y));
        object.numbers(ACCEL_KEY, &self.accel.map(round_hundredths));
        object.number(GYRO_ROLL_KEY, round_hundredths(self.gyro_roll));
        object.numbers(ROTATION_KEY, &rotation);
        object.number(FACING_KEY, round_hundredths(self.facing));
        object.boolean(RECENTER_KEY, self.has_recentered);
        object.boolean(MARK_KEY, self.is_marked);
        object.finish()
    }

    /// One line as the bridge wrote it, in any version: `facing` may be missing (read as 1),
    /// whitespace and unknown keys are ignored; `None` when the line is not such a record.
    pub fn parse_line(line: &str) -> Option<Self> {
        let members = Members(parse_flat_object(line)?);
        Some(Self {
            timestamp_ms: members.number(TIMESTAMP_KEY)?,
            pitch: members.number(PITCH_KEY)?,
            yaw: members.number(YAW_KEY)?,
            pointer_x: members.number(POINTER_X_KEY)?,
            pointer_y: members.number(POINTER_Y_KEY)?,
            accel: members.array(ACCEL_KEY)?,
            gyro_roll: members.number(GYRO_ROLL_KEY)?,
            rotation: members.rotation()?,
            facing: members.facing()?,
            has_recentered: members.boolean(RECENTER_KEY)?,
            is_marked: members.boolean(MARK_KEY)?,
        })
    }
}

/// A line the TypeScript bridge wrote; [`crate::session::TelemetrySample`]'s test writes it too.
#[cfg(test)]
pub(crate) const GOLDEN_LINE: &str = r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"facing":0.99,"recenter":true,"mark":false}"#;

#[cfg(test)]
mod tests;
