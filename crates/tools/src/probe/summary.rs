//! Running min / max of what the probe has seen.

use std::fmt;

use dsu::{Buttons1, Buttons2};

use crate::probe::PadReading;
use numeric::format::to_fixed;

/// The range of one stick axis byte seen so far.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AxisRange {
    low: u8,
    high: u8,
}

impl AxisRange {
    fn widened(range: Option<Self>, value: u8) -> Self {
        range.map_or(
            Self {
                low: value,
                high: value,
            },
            |seen| Self {
                low: seen.low.min(value),
                high: seen.high.max(value),
            },
        )
    }
}

/// Pointer range, peak gyro and every button seen over the probe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProbeSummary {
    packets: usize,
    pointer_x: Option<AxisRange>,
    pointer_y: Option<AxisRange>,
    gyro: f64,
    buttons1: Buttons1,
    buttons2: Buttons2,
}

impl ProbeSummary {
    pub fn add(&mut self, reading: &PadReading) {
        self.packets += 1;
        self.pointer_x = Some(AxisRange::widened(self.pointer_x, reading.pointer.x));
        self.pointer_y = Some(AxisRange::widened(self.pointer_y, reading.pointer.y));
        self.gyro = self.gyro.max(reading.peak_gyro);
        self.buttons1 |= reading.buttons1;
        self.buttons2 |= reading.buttons2;
    }

    /// Whether no pad packet arrived.
    pub fn is_empty(&self) -> bool {
        self.packets == 0
    }
}

impl fmt::Display for ProbeSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x_range = self.pointer_x.unwrap_or_default();
        let y_range = self.pointer_y.unwrap_or_default();
        let buttons = u16::from_le_bytes([self.buttons1.0, self.buttons2.0]);
        write!(
            formatter,
            "{} packets | pointer X {}-{}  Y {}-{} (128 = centre) | max gyro {} deg/s | buttons seen 0x{buttons:04x}",
            self.packets,
            x_range.low,
            x_range.high,
            y_range.low,
            y_range.high,
            to_fixed(self.gyro, 0),
        )
    }
}

#[cfg(test)]
mod tests {
    use dsu::{Buttons1, Buttons2, Stick};

    use crate::probe::{PadReading, ProbeSummary};

    /// (pointer x, peak gyro, first button byte).
    const READINGS: [(u8, f64, u8); 2] = [(120, 10.4, 0x01), (140, 250.6, 0x80)];

    #[test]
    fn summarises_readings() {
        let mut summary = ProbeSummary::default();
        for (x, peak_gyro, buttons) in READINGS {
            summary.add(&PadReading {
                pointer: Stick {
                    x,
                    y: Stick::CENTRE.y,
                },
                buttons1: Buttons1(buttons),
                buttons2: Buttons2::NONE,
                peak_gyro,
            });
        }
        assert_eq!(
            summary.to_string(),
            "2 packets | pointer X 120-140  Y 128-128 (128 = centre) | max gyro 251 deg/s | buttons seen 0x0081"
        );
    }
}
