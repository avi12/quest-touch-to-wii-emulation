use std::time::Duration;

use crate::pointer::PointerPosition;

const SWEEP_STEP: Duration = Duration::from_secs(4);
const POINTER_SWEEP: [PointerPosition; 5] = [
    PointerPosition { x: 0.0, y: 0.0 },
    PointerPosition { x: 1.0, y: 0.0 },
    PointerPosition { x: -1.0, y: 0.0 },
    PointerPosition { x: 0.0, y: 1.0 },
    PointerPosition { x: 0.0, y: -1.0 },
];

/// Known pointer values, ignoring the controller (`PTR_TEST` diagnostic). `since_epoch` is the
/// wall clock since the Unix epoch, so every bridge shows the same step at the same moment.
#[must_use]
pub fn sweep_pointer(since_epoch: Duration) -> PointerPosition {
    let step =
        usize::try_from(since_epoch.as_millis() / SWEEP_STEP.as_millis()).unwrap_or_default();
    POINTER_SWEEP
        .get(step % POINTER_SWEEP.len())
        .copied()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::pointer::sweep::{SWEEP_STEP, sweep_pointer};

    const LAST_STEP: u32 = 4;
    const CYCLE_STEPS: u32 = 5;

    #[test]
    fn walks_the_sweep_every_four_seconds() {
        let just_before = SWEEP_STEP.saturating_sub(Duration::from_millis(1));
        assert_eq!(sweep_pointer(just_before).x, 0.0);
        assert_eq!(sweep_pointer(SWEEP_STEP).x, 1.0);
        assert_eq!(sweep_pointer(SWEEP_STEP * LAST_STEP).y, -1.0);
        assert_eq!(sweep_pointer(SWEEP_STEP * CYCLE_STEPS).y, 0.0);
    }
}
