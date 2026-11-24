use std::f64::consts::TAU;
use std::time::Duration;

const DEFAULT_MIN_CUTOFF: f64 = 0.8;
const DEFAULT_BETA: f64 = 0.6;
const DEFAULT_DERIVATIVE_CUTOFF: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OneEuroOptions {
    pub min_cutoff: f64,
    pub beta: f64,
    pub derivative_cutoff: f64,
}

impl Default for OneEuroOptions {
    fn default() -> Self {
        Self {
            min_cutoff: DEFAULT_MIN_CUTOFF,
            beta: DEFAULT_BETA,
            derivative_cutoff: DEFAULT_DERIVATIVE_CUTOFF,
        }
    }
}

/// One-euro filter: smooths jitter when slow, stays responsive when fast.
#[derive(Clone, Debug, Default)]
pub struct OneEuro {
    value: Option<f64>,
    derivative: f64,
    options: OneEuroOptions,
}

fn alpha(cutoff: f64, delta_seconds: f64) -> f64 {
    1.0 / (1.0 + 1.0 / (TAU * cutoff * delta_seconds))
}

impl OneEuro {
    #[must_use]
    pub fn new(options: OneEuroOptions) -> Self {
        Self {
            value: None,
            derivative: 0.0,
            options,
        }
    }

    /// The last filtered value, `None` until the first sample and after a reset.
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    pub fn reset(&mut self) {
        self.value = None;
        self.derivative = 0.0;
    }

    /// Smooths `value`, sampled `delta` after the previous one.
    pub fn filter(&mut self, value: f64, delta: Duration) -> f64 {
        let delta_seconds = delta.as_secs_f64();
        let Some(previous) = self.value else {
            self.value = Some(value);
            return value;
        };
        let speed = (value - previous) / delta_seconds;
        self.derivative +=
            alpha(self.options.derivative_cutoff, delta_seconds) * (speed - self.derivative);
        let cutoff = self.options.min_cutoff + self.options.beta * self.derivative.abs();
        let filtered = previous + alpha(cutoff, delta_seconds) * (value - previous);
        self.value = Some(filtered);
        filtered
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::math::{OneEuro, OneEuroOptions};

    const STEP: Duration = Duration::from_millis(4);
    const FIRST_SAMPLE: f64 = 0.5;
    const TOLERANCE: f64 = 1e-12;
    // From src/math.ts: filter(0), then filter(1) twice, at 4 ms steps.
    const GOLDEN_SECOND: f64 = 0.101_149_041_316_574_08;
    const GOLDEN_THIRD: f64 = 0.246_778_481_259_572_02;

    #[test]
    fn first_sample_passes_through_and_reset_clears() {
        let mut filter = OneEuro::default();
        assert_eq!(filter.filter(FIRST_SAMPLE, STEP), 0.5);
        assert_eq!(filter.value(), Some(0.5));
        filter.reset();
        assert_eq!(filter.value(), None);
    }

    #[test]
    fn matches_typescript_on_a_step() {
        let mut filter = OneEuro::new(OneEuroOptions::default());
        filter.filter(0.0, STEP);
        let second = filter.filter(1.0, STEP);
        let third = filter.filter(1.0, STEP);
        assert!((second - GOLDEN_SECOND).abs() < TOLERANCE, "{second}");
        assert!((third - GOLDEN_THIRD).abs() < TOLERANCE, "{third}");
    }
}
