//! The printed lines: summary, findings, pointing-axis scores and 250 ms rows.

use bridge::math::{Vec3, dot, norm, radians_to_degrees};
use bridge::pointer::CANDIDATES;

use super::{MS_PER_SECOND, Sample};
use crate::format::{js_number, to_fixed};
use crate::stats::{percent, spread_text};

const EDGE: f64 = 0.98;
const ROW_MS: f64 = 250.0;
/// Aim this far above or below level (deg): roll is ill-defined there.
const STEEP_PITCH: f64 = 70.0;
/// Pointer movement (-1..1) in one 20 ms sample that is a jump, not a movement.
const JUMP: f64 = 0.25;
/// Acceleration (g) above which a sample counts as a swing.
const SWING_G: f64 = 2.0;
const CANDIDATE_NAME_WIDTH: usize = 28;
const UP_ROW: std::ops::Range<usize> = 3..6;

fn series(window: &[Sample], field: fn(&Sample) -> f64) -> String {
    spread_text(&window.iter().map(field).collect::<Vec<_>>())
}

fn count(window: &[Sample], predicate: impl Fn(&Sample) -> bool) -> usize {
    window.iter().filter(|sample| predicate(sample)).count()
}

/// The six summary lines for `window`, headed by `label`.
pub fn summary_lines(window: &[Sample], label: &str) -> Vec<String> {
    let edge = count(window, |sample| {
        sample.pointer_x.abs() >= EDGE || sample.pointer_y.abs() >= EDGE
    });
    let recenters = count(window, |sample| sample.recenter);
    let accel_max = window
        .iter()
        .map(|sample| norm(sample.accel))
        .fold(f64::NEG_INFINITY, f64::max);
    vec![
        format!("== {label}: {} samples", window.len()),
        format!("aim pitch deg {}", series(window, |sample| sample.pitch)),
        format!("aim yaw   deg {}", series(window, |sample| sample.yaw)),
        format!(
            "pointer x {} | y {}",
            series(window, |sample| sample.pointer_x),
            series(window, |sample| sample.pointer_y)
        ),
        format!(
            "pointer at an edge {}% | recenters {recenters}",
            to_fixed(percent(edge, window.len()), 0)
        ),
        format!(
            "accel g max {} | roll rate deg/s {}",
            to_fixed(accel_max, 2),
            series(window, |sample| sample.gyro_roll)
        ),
    ]
}

/// Plain-language findings for one recording.
pub fn diagnostics_line(take: &[Sample]) -> String {
    let steep = count(take, |sample| sample.pitch.abs() > STEEP_PITCH);
    let jumps = take
        .windows(2)
        .filter(|pair| match pair {
            [previous, current] => {
                (current.pointer_x - previous.pointer_x)
                    .hypot(current.pointer_y - previous.pointer_y)
                    > JUMP
            }
            _ => false,
        })
        .count();
    let swings = count(take, |sample| norm(sample.accel) > SWING_G);
    format!(
        "findings: aim steeper than {} deg {}% of the time | pointer jumps >{} in 20 ms: {jumps} | samples over {} g: {swings}",
        js_number(STEEP_PITCH),
        to_fixed(percent(steep, take.len()), 0),
        js_number(JUMP),
        js_number(SWING_G),
    )
}

/// Pitch (deg above level) of a controller-frame axis, from the logged rotation.
fn axis_pitch(rotation: &[f64; 9], axis: Vec3) -> f64 {
    let up_row = match rotation.get(UP_ROW) {
        Some([row_x, row_y, row_z]) => [*row_x, *row_y, *row_z],
        _ => [0.0; 3],
    };
    radians_to_degrees(dot(up_row, axis).clamp(-1.0, 1.0).asin())
}

/// How steady each pointing-axis candidate would have been: smaller swing is better.
pub fn axis_score_lines(take: &[Sample]) -> Vec<String> {
    let header = "pointing-axis candidates (pitch swing; smaller is steadier):".to_owned();
    let scores = CANDIDATES.iter().map(|candidate| {
        let pitches: Vec<f64> = take
            .iter()
            .map(|sample| axis_pitch(&sample.rotation, candidate.axis))
            .collect();
        format!(
            "  {:<CANDIDATE_NAME_WIDTH$} pitch {}",
            candidate.name,
            spread_text(&pitches)
        )
    });
    std::iter::once(header).chain(scores).collect()
}

/// One row per 250 ms, timed from `origin` (ms).
pub fn row_lines(window: &[Sample], origin: f64) -> Vec<String> {
    let mut next_row = 0.0;
    let mut lines = Vec::new();
    for sample in window {
        if sample.timestamp < next_row {
            continue;
        }
        next_row = sample.timestamp + ROW_MS;
        let accel: Vec<String> = sample.accel.iter().map(|value| js_number(*value)).collect();
        lines.push(format!(
            "{}s pitch {} yaw {} ptr({}, {}) accel({})",
            to_fixed((sample.timestamp - origin) / MS_PER_SECOND, 2),
            js_number(sample.pitch),
            js_number(sample.yaw),
            js_number(sample.pointer_x),
            js_number(sample.pointer_y),
            accel.join(", "),
        ));
    }
    lines
}
