//! How steady each pointing-axis candidate would have been over a recording.

use bridge::math::{Degrees, Radians, dot};
use bridge::pointer::CANDIDATES;
use bridge::session::record::Record;
use geometry::{Mat3, Vec3};

use crate::stats::spread_text;

const CANDIDATE_NAME_WIDTH: usize = 28;

/// Pitch above level of a controller-frame axis, from the logged rotation.
fn axis_pitch(rotation: Mat3, axis: Vec3) -> Degrees {
    let [_, up_row, _] = rotation;
    Radians(dot(up_row, axis).clamp(-1.0, 1.0).asin()).to_degrees()
}

/// The candidate lines for `take`: smaller swing is steadier.
pub fn axis_score_lines(take: &[&Record]) -> Vec<String> {
    let header = "pointing-axis candidates (pitch swing; smaller is steadier):".to_owned();
    let scores = CANDIDATES.iter().map(|candidate| {
        let pitches: Vec<f64> = take
            .iter()
            .map(|record| axis_pitch(record.rotation, candidate.axis).0)
            .collect();
        format!(
            "  {:<CANDIDATE_NAME_WIDTH$} pitch {}",
            candidate.name,
            spread_text(&pitches)
        )
    });
    std::iter::once(header).chain(scores).collect()
}
