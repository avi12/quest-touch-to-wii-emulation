//! Plain-language findings for one recording.

use bridge::math::{Degrees, norm};
use bridge::session::record::Record;

use crate::stats::Percent;
use crate::telemetry::report::series::count;
use numeric::format::js_number;

/// Aim this far above or below level: roll is ill-defined there.
const STEEP_PITCH: Degrees = Degrees(70.0);
/// Pointer movement (-1..1) in one 20 ms sample that is a jump, not a movement.
const JUMP: f64 = 0.25;
/// Acceleration (g) above which a sample counts as a swing.
const SWING_G: f64 = 2.0;

/// Whether the pointer moved further than [`JUMP`] from `previous` to `current`.
fn is_jump(previous: &Record, current: &Record) -> bool {
    let distance =
        (current.pointer_x - previous.pointer_x).hypot(current.pointer_y - previous.pointer_y);
    distance > JUMP
}

/// The findings line for `take`.
pub fn diagnostics_line(take: &[&Record]) -> String {
    let steep = count(take, |record| Degrees(record.pitch.abs()) > STEEP_PITCH);
    let jumps = take
        .windows(2)
        .filter(|pair| matches!(pair, [previous, current] if is_jump(previous, current)))
        .count();
    let swings = count(take, |record| norm(record.accel) > SWING_G);
    format!(
        "findings: aim steeper than {} deg {}% of the time | pointer jumps >{} in 20 ms: {jumps} | samples over {} g: {swings}",
        js_number(STEEP_PITCH.0),
        Percent::share(steep, take.len()).whole_text(),
        js_number(JUMP),
        js_number(SWING_G),
    )
}
