//! Recordings: each stretch of the right controller's B button held.

use super::Sample;

/// B released shorter than this (ms) still counts as the same recording.
const TAKE_GAP_MS: f64 = 200.0;

/// Each unbroken stretch of B held is one recording.
pub fn split_takes(samples: &[Sample]) -> Vec<Vec<Sample>> {
    let mut takes: Vec<Vec<Sample>> = Vec::new();
    let mut last_held_at = 0.0;
    for sample in samples.iter().filter(|sample| sample.mark) {
        let is_new_take = sample.timestamp - last_held_at > TAKE_GAP_MS;
        match takes.last_mut() {
            Some(take) if !is_new_take => take.push(*sample),
            _ => takes.push(vec![*sample]),
        }
        last_held_at = sample.timestamp;
    }
    takes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(timestamp: f64, mark: bool) -> Sample {
        Sample {
            timestamp,
            pitch: 0.0,
            yaw: 0.0,
            pointer_x: 0.0,
            pointer_y: 0.0,
            accel: [0.0; 3],
            gyro_roll: 0.0,
            rotation: [0.0; 9],
            facing: 1.0,
            recenter: false,
            mark,
        }
    }

    #[test]
    fn splits_on_gaps_longer_than_the_tolerance() {
        let samples = [
            sample(1000.0, true),
            sample(1020.0, true),
            sample(1100.0, false),
            sample(1200.0, true), // 180 ms after the last held sample: same take
            sample(1500.0, true), // 300 ms: a new take
            sample(1600.0, false),
        ];
        let takes = split_takes(&samples);
        assert_eq!(takes.iter().map(Vec::len).collect::<Vec<_>>(), [3, 1]);
    }
}
