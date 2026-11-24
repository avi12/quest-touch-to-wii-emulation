use std::time::Duration;

use super::PointerPosition;

const SWEEP_STEP_MS: u128 = 4000;
const POINTER_SWEEP: [[f64; 2]; 5] = [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]];

/// Known pointer values, ignoring the controller (`PTR_TEST` diagnostic). `since_epoch` is the
/// wall clock since the Unix epoch, so every bridge shows the same step at the same moment.
#[must_use]
pub fn sweep_pointer(since_epoch: Duration) -> PointerPosition {
    let step = usize::try_from(since_epoch.as_millis() / SWEEP_STEP_MS).unwrap_or_default();
    let [x, y] = POINTER_SWEEP
        .get(step % POINTER_SWEEP.len())
        .copied()
        .unwrap_or_default();
    PointerPosition { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_the_sweep_every_four_seconds() {
        assert_eq!(sweep_pointer(Duration::from_millis(3999)).x, 0.0);
        assert_eq!(sweep_pointer(Duration::from_secs(4)).x, 1.0);
        assert_eq!(sweep_pointer(Duration::from_secs(16)).y, -1.0);
        assert_eq!(sweep_pointer(Duration::from_secs(20)).y, 0.0);
    }
}
