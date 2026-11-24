use std::f64::consts::FRAC_1_SQRT_2;

use crate::session::record::{GOLDEN_LINE, Record};

/// [`GOLDEN_LINE`] as read back.
const GOLDEN_RECORD: Record = Record {
    timestamp_ms: 1234.0,
    pitch: 11.9,
    yaw: 18.43,
    pointer_x: 0.13,
    pointer_y: -0.33,
    accel: [0.0, -1.0, 0.26],
    gyro_roll: -12.34,
    rotation: [[1.0, 0.0, 0.0], [0.0, 0.71, -0.71], [0.0, 0.71, 0.71]],
    facing: 0.99,
    has_recentered: true,
    is_marked: false,
};

/// Unrounded values, as the bridge holds them before writing.
const RAW_RECORD: Record = Record {
    timestamp_ms: 1_700_000_000_123.0,
    pitch: -20.784,
    yaw: 179.999,
    pointer_x: 0.95,
    pointer_y: -0.004,
    accel: [0.006, -1.0, 2.5],
    gyro_roll: 78.75,
    rotation: [
        [1.0, 0.0, 0.0],
        [0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2],
        [0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
    ],
    facing: -0.5,
    has_recentered: false,
    is_marked: true,
};

/// An older line: no `facing`, an unknown key, pretty-printed.
const OLD_LINE: &str = r#" { "timestamp": 1, "pitch": -20.78, "yaw": -85.9, "pointerX": 0, "pointerY": 0,
    "accel": [0.01, -1, 0], "gyroRoll": 78.75, "rotation": [1, 0, 0, 0, 1, 0, 0, 0, 1],
    "recenter": true, "mark": false, "extra": "note", "spare": null } "#;

const MALFORMED_LINES: [&str; 10] = [
    "",
    "[]",
    r#"{"timestamp":1}"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"recenter":true,"mark":false}"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71],"recenter":true,"mark":false}"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"recenter":1,"mark":false}"#,
    r#"{"timestamp":1234,"pitch":"11.9","yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"recenter":true,"mark":false}"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"facing":true,"recenter":true,"mark":false}"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"recenter":true,"mark":false} trailing"#,
    r#"{"timestamp":1234,"pitch":11.9,"yaw":18.43,"pointerX":0.13,"pointerY":-0.33,"accel":[0,-1,0.26],"gyroRoll":-12.34,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"recenter":true,"mark":false,"nested":{"a":1}}"#,
];

#[test]
fn reads_the_golden_line() {
    assert_eq!(Record::parse_line(GOLDEN_LINE), Some(GOLDEN_RECORD));
}

#[test]
fn writes_the_golden_line() {
    assert_eq!(GOLDEN_RECORD.to_json_line(), GOLDEN_LINE);
}

#[test]
fn round_trips_through_rounding() {
    let line = RAW_RECORD.to_json_line();
    assert_eq!(
        line,
        r#"{"timestamp":1700000000123,"pitch":-20.78,"yaw":180,"pointerX":0.95,"pointerY":0,"accel":[0.01,-1,2.5],"gyroRoll":78.75,"rotation":[1,0,0,0,0.71,-0.71,0,0.71,0.71],"facing":-0.5,"recenter":false,"mark":true}"#
    );
    let read = Record::parse_line(&line).unwrap();
    assert_eq!(read.to_json_line(), line);
    assert!(read.is_marked && !read.has_recentered);
}

#[test]
fn reads_older_lines_with_defaults() {
    let record = Record::parse_line(OLD_LINE).unwrap();
    assert_eq!(record.facing, 1.0);
    assert_eq!(record.yaw, -85.9);
    assert_eq!(record.rotation[1], [0.0, 1.0, 0.0]);
    assert!(record.has_recentered);
}

#[test]
fn null_reads_as_nan() {
    let line = GOLDEN_LINE.replace("\"pitch\":11.9", "\"pitch\":null");
    let record = Record::parse_line(&line).unwrap();
    assert!(record.pitch.is_nan());
    assert!(record.to_json_line().contains("\"pitch\":null"));
}

#[test]
fn rejects_malformed_lines() {
    for line in MALFORMED_LINES {
        assert_eq!(Record::parse_line(line), None, "{line}");
    }
}
