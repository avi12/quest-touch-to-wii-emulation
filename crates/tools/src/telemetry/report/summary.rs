//! The six summary lines of a window: aim, pointer, edges, recentres, motion.

use bridge::math::norm;
use bridge::session::record::Record;

use crate::stats::Percent;
use crate::telemetry::report::series::{count, series};
use numeric::format::to_fixed;

/// Pointer magnitude (-1..1) from which it counts as sitting at an edge.
const EDGE: f64 = 0.98;
const ACCEL_DECIMALS: usize = 2;

/// The six summary lines for `window`, headed by `label`.
pub fn summary_lines(window: &[&Record], label: &str) -> Vec<String> {
    let edge = count(window, |record| {
        record.pointer_x.abs() >= EDGE || record.pointer_y.abs() >= EDGE
    });
    let recenters = count(window, |record| record.has_recentered);
    let accel_max = window
        .iter()
        .map(|record| norm(record.accel))
        .fold(f64::NEG_INFINITY, f64::max);
    vec![
        format!("== {label}: {} samples", window.len()),
        format!("aim pitch deg {}", series(window, |record| record.pitch)),
        format!("aim yaw   deg {}", series(window, |record| record.yaw)),
        format!(
            "pointer x {} | y {}",
            series(window, |record| record.pointer_x),
            series(window, |record| record.pointer_y)
        ),
        format!(
            "pointer at an edge {}% | recenters {recenters}",
            Percent::share(edge, window.len()).whole_text()
        ),
        format!(
            "accel g max {} | roll rate deg/s {}",
            to_fixed(accel_max, ACCEL_DECIMALS),
            series(window, |record| record.gyro_roll)
        ),
    ]
}
