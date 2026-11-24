//! Thumbstick dead zone: a Touch stick at rest reports a few hundredths off centre, which Dolphin
//! reads as a slight push. Inside the dead zone the stick is exactly (0, 0); beyond it the travel
//! is rescaled so the output still runs smoothly from 0 to 1.

/// Stick deflection (0..1, radial) below which the stick counts as centred.
const DEAD_ZONE: f64 = 0.15;
const FULL_DEFLECTION: f64 = 1.0;

/// `stick` (x, y in -1..1) with the radial dead zone removed.
pub(crate) fn remove_dead_zone(stick: [f64; 2]) -> [f64; 2] {
    let [x, y] = stick;
    let deflection = x.hypot(y);
    let is_resting = deflection <= DEAD_ZONE;
    if is_resting {
        return [0.0, 0.0];
    }
    let rescaled = ((deflection - DEAD_ZONE) / (FULL_DEFLECTION - DEAD_ZONE)).min(FULL_DEFLECTION);
    let factor = rescaled / deflection;
    [x * factor, y * factor]
}

#[cfg(test)]
mod tests {
    use crate::input::deadzone::remove_dead_zone;

    const DRIFT: f64 = 0.06;
    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn resting_drift_reads_as_centre() {
        assert_eq!(remove_dead_zone([DRIFT, -DRIFT]), [0.0, 0.0]);
    }

    #[test]
    fn full_push_stays_full() {
        let [x, y] = remove_dead_zone([0.0, 1.0]);
        assert!(x.abs() < TOLERANCE && (y - 1.0).abs() < TOLERANCE);
        let [left, _] = remove_dead_zone([-1.0, 0.0]);
        assert!((left + 1.0).abs() < TOLERANCE);
    }
}
