//! Running min / max of what the probe has seen.

use std::fmt;

use super::PadReading;
use crate::format::to_fixed;

/// Pointer range, peak gyro and every button seen over the probe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProbeSummary {
    packets: usize,
    pointer_x: Option<(u8, u8)>,
    pointer_y: Option<(u8, u8)>,
    gyro: f64,
    buttons: u16,
}

impl ProbeSummary {
    pub fn add(&mut self, reading: &PadReading) {
        self.packets += 1;
        self.pointer_x = Some(widen(self.pointer_x, reading.pointer_x));
        self.pointer_y = Some(widen(self.pointer_y, reading.pointer_y));
        self.gyro = self.gyro.max(reading.peak_gyro);
        self.buttons |= reading.buttons;
    }

    pub fn packets(&self) -> usize {
        self.packets
    }
}

fn widen(range: Option<(u8, u8)>, value: u8) -> (u8, u8) {
    range.map_or((value, value), |(low, high)| {
        (low.min(value), high.max(value))
    })
}

impl fmt::Display for ProbeSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x_low, x_high) = self.pointer_x.unwrap_or_default();
        let (y_low, y_high) = self.pointer_y.unwrap_or_default();
        write!(
            formatter,
            "{} packets | pointer X {x_low}-{x_high}  Y {y_low}-{y_high} (128 = centre) | max gyro {} deg/s | buttons seen 0x{:04x}",
            self.packets,
            to_fixed(self.gyro, 0),
            self.buttons,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarises_readings() {
        let mut summary = ProbeSummary::default();
        for (x, peak_gyro, buttons) in [(120, 10.4, 0x01), (140, 250.6, 0x80)] {
            summary.add(&PadReading {
                pointer_x: x,
                pointer_y: 128,
                buttons,
                peak_gyro,
            });
        }
        assert_eq!(
            summary.to_string(),
            "2 packets | pointer X 120-140  Y 128-128 (128 = centre) | max gyro 251 deg/s | buttons seen 0x0081"
        );
    }
}
