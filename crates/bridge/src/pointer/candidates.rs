use crate::math::Vec3;

/// A pointing axis to try live with Ctrl+Alt+[ ].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub name: &'static str,
    /// Axis in the controller frame.
    pub axis: Vec3,
}

/// Pointing axes to try live with Ctrl+Alt+[ ].
pub const CANDIDATES: [Candidate; 9] = [
    Candidate {
        name: "measured twist axis",
        axis: [-0.134, 0.991, 0.025],
    },
    Candidate {
        name: "+x",
        axis: [1.0, 0.0, 0.0],
    },
    Candidate {
        name: "-x",
        axis: [-1.0, 0.0, 0.0],
    },
    Candidate {
        name: "+y",
        axis: [0.0, 1.0, 0.0],
    },
    Candidate {
        name: "-y",
        axis: [0.0, -1.0, 0.0],
    },
    Candidate {
        name: "+z",
        axis: [0.0, 0.0, 1.0],
    },
    Candidate {
        name: "-z",
        axis: [0.0, 0.0, -1.0],
    },
    Candidate {
        name: "Meta aim ray (0,-.61,-.79)",
        axis: [0.0, -0.607, -0.794],
    },
    Candidate {
        name: "Meta aim ray reversed",
        axis: [0.0, 0.607, 0.794],
    },
];

/// "Meta aim ray": measured level (7 deg) when holding still aimed at the screen; -z was 44 deg up.
pub const DEFAULT_CANDIDATE_INDEX: usize = 7;

/// The candidate index `step` places away from `index`, wrapping around both ends.
pub(crate) fn step_candidate(index: usize, step: isize) -> usize {
    let count = CANDIDATES.len();
    let distance = step.unsigned_abs() % count;
    if step >= 0 {
        return (index + distance) % count;
    }
    (index + count - distance) % count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_wrap_both_ways() {
        assert_eq!(step_candidate(8, 1), 0);
        assert_eq!(step_candidate(0, -1), 8);
        assert_eq!(step_candidate(DEFAULT_CANDIDATE_INDEX, 0), 7);
    }
}
