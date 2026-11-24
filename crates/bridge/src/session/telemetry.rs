//! Optional live stream of what the bridge sees and sends, as JSON lines, so the motion can be
//! inspected while it happens. Each line is a [`Record`], which `tools/telemetry_tail` reads back.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use crate::math::{Mat3, Radians, Vec3, norm};
use crate::motion::Sensors;
use crate::pointer::PointerPosition;
use crate::session::record::Record;

/// 50 samples per second.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(20);
const MS_PER_SECOND: f64 = 1000.0;

/// One frame as the telemetry records it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TelemetrySample {
    /// Where the aim axis points in the world.
    pub direction: Vec3,
    pub pointer: PointerPosition,
    pub sensors: Sensors,
    pub has_recentered: bool,
    pub rotation: Mat3,
    pub facing: f64,
    /// B held: the person is recording a take to analyse.
    pub is_marked: bool,
}

/// Milliseconds since the Unix epoch, exact for the whole-millisecond wall clock.
fn epoch_milliseconds(wall_clock: Duration) -> f64 {
    let seconds = u32::try_from(wall_clock.as_secs()).map_or(f64::from(u32::MAX), f64::from);
    seconds * MS_PER_SECOND + f64::from(wall_clock.subsec_millis())
}

impl TelemetrySample {
    /// The record written for this sample at `wall_clock` (since the Unix epoch).
    pub fn to_record(&self, wall_clock: Duration) -> Record {
        let [east, height, south] = self.direction;
        let pitch = Radians((height / norm(self.direction)).asin()).to_degrees();
        let yaw = Radians(east.atan2(-south)).to_degrees();
        let [_, _, gyro_roll] = self.sensors.gyro.0;
        Record {
            timestamp_ms: epoch_milliseconds(wall_clock),
            pitch: pitch.0,
            yaw: yaw.0,
            pointer_x: self.pointer.x,
            pointer_y: self.pointer.y,
            accel: self.sensors.accel.0,
            gyro_roll,
            rotation: self.rotation,
            facing: self.facing,
            has_recentered: self.has_recentered,
            is_marked: self.is_marked,
        }
    }
}

/// Appends one line per sample to a file; does nothing when no path is given.
#[derive(Debug)]
pub struct Telemetry {
    file: Option<File>,
    /// Wall clock of the last line written.
    last_written_at: Duration,
}

impl Telemetry {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        let file = path
            .map(|path| OpenOptions::new().create(true).append(true).open(path))
            .transpose()?;
        Ok(Self {
            file,
            last_written_at: Duration::ZERO,
        })
    }

    /// Writes the sample when [`SAMPLE_INTERVAL`] has passed since the last one, or on a recenter.
    pub fn record(&mut self, sample: &TelemetrySample, wall_clock: Duration) -> io::Result<()> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };
        let is_due = wall_clock.saturating_sub(self.last_written_at) >= SAMPLE_INTERVAL;
        let is_skipped = !is_due && !sample.has_recentered;
        if is_skipped {
            return Ok(());
        }
        self.last_written_at = wall_clock;
        let line = sample.to_record(wall_clock).to_json_line();
        file.write_all(format!("{line}\n").as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_1_SQRT_2;
    use std::time::Duration;

    use crate::math::{DegreesPerSecond, Gravities};
    use crate::motion::Sensors;
    use crate::pointer::PointerPosition;
    use crate::session::TelemetrySample;
    use crate::session::record::GOLDEN_LINE;

    const WALL_CLOCK: Duration = Duration::from_millis(1234);
    const SAMPLE: TelemetrySample = TelemetrySample {
        direction: [0.3, 0.2, -0.9],
        pointer: PointerPosition {
            x: 0.125,
            y: -0.335,
        },
        sensors: Sensors {
            accel: Gravities([-0.004, -1.0, 0.256]),
            gyro: DegreesPerSecond([1.0, 2.0, -12.345]),
        },
        has_recentered: true,
        rotation: [
            [1.0, 0.0, 0.0],
            [0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2],
            [0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
        ],
        facing: 0.98765,
        is_marked: false,
    };

    /// The TypeScript `Telemetry` wrote [`GOLDEN_LINE`] for the same sample.
    #[test]
    fn matches_typescript() {
        assert_eq!(SAMPLE.to_record(WALL_CLOCK).to_json_line(), GOLDEN_LINE);
    }
}
