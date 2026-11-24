//! Recordings: each stretch of the right controller's B button held.

use std::time::Duration;

use bridge::session::record::Record;

use crate::telemetry::millis;

/// B released shorter than this still counts as the same recording.
const TAKE_GAP: Duration = Duration::from_millis(200);

/// Each unbroken stretch of B held is one recording.
pub fn split_takes(records: &[Record]) -> Vec<Vec<&Record>> {
    let gap_ms = millis(TAKE_GAP);
    let mut takes: Vec<Vec<&Record>> = Vec::new();
    let mut last_held_at = 0.0;
    for record in records.iter().filter(|record| record.is_marked) {
        let is_new_take = record.timestamp_ms - last_held_at > gap_ms;
        match takes.last_mut() {
            Some(take) if !is_new_take => take.push(record),
            _ => takes.push(vec![record]),
        }
        last_held_at = record.timestamp_ms;
    }
    takes
}

#[cfg(test)]
mod tests {
    use bridge::session::record::Record;
    use geometry::Mat3;

    use crate::telemetry::split_takes;

    /// (timestamp ms, B held): 180 ms after the last held sample is the same take, 300 ms a new
    /// one.
    const STREAM: [(f64, bool); 6] = [
        (1000.0, true),
        (1020.0, true),
        (1100.0, false),
        (1200.0, true),
        (1500.0, true),
        (1600.0, false),
    ];
    const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    fn record((timestamp_ms, is_marked): (f64, bool)) -> Record {
        Record {
            timestamp_ms,
            pitch: 0.0,
            yaw: 0.0,
            pointer_x: 0.0,
            pointer_y: 0.0,
            accel: [0.0; 3],
            gyro_roll: 0.0,
            rotation: IDENTITY,
            facing: 1.0,
            has_recentered: false,
            is_marked,
        }
    }

    #[test]
    fn splits_on_gaps_longer_than_the_tolerance() {
        let records = STREAM.map(record);
        let takes = split_takes(&records);
        assert_eq!(takes.iter().map(Vec::len).collect::<Vec<_>>(), [3, 1]);
    }
}
