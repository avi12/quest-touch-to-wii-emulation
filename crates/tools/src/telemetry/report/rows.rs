//! One printed row per 250 ms of a window.

use std::time::Duration;

use bridge::session::record::Record;

use crate::telemetry::{elapsed_seconds, millis};
use numeric::format::{js_number, to_fixed};

const ROW_EVERY: Duration = Duration::from_millis(250);
const OFFSET_DECIMALS: usize = 2;

fn row(record: &Record, origin_ms: f64) -> String {
    let accel: Vec<String> = record.accel.iter().map(|value| js_number(*value)).collect();
    format!(
        "{}s pitch {} yaw {} ptr({}, {}) accel({})",
        to_fixed(
            elapsed_seconds(origin_ms, record.timestamp_ms),
            OFFSET_DECIMALS
        ),
        js_number(record.pitch),
        js_number(record.yaw),
        js_number(record.pointer_x),
        js_number(record.pointer_y),
        accel.join(", "),
    )
}

/// One row per 250 ms, timed from `origin_ms`.
pub fn row_lines(window: &[&Record], origin_ms: f64) -> Vec<String> {
    let row_ms = millis(ROW_EVERY);
    let mut next_row_ms = 0.0;
    let mut lines = Vec::new();
    for record in window {
        let is_too_soon = record.timestamp_ms < next_row_ms;
        if is_too_soon {
            continue;
        }
        next_row_ms = record.timestamp_ms + row_ms;
        lines.push(row(record, origin_ms));
    }
    lines
}
