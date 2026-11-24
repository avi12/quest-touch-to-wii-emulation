//! Golden values from running src/pointer.ts on the same frames.

use std::time::Duration;

use crate::math::rotation_from_angles;
use crate::pointer::{CandidateStep, Pointer, PointerInput};

const TOLERANCE: f64 = 1e-9;
const FRAME: Duration = Duration::from_millis(4);

/// yaw, pitch, roll, thumbstick click.
const FRAMES: [(f64, f64, f64, bool); 6] = [
    (0.0, 0.0, 0.0, false),
    (0.05, 0.02, 0.0, false),
    (0.1, -0.04, 0.3, false),
    (0.12, 0.05, 0.3, true),
    (0.2, 0.0, 0.0, false),
    (1.6, 0.0, 0.0, false),
];

/// x, y, did recenter, is aimed away, facing.
const EXPECTED: [(f64, f64, bool, bool, f64); 6] = [
    (0.0, 0.0, true, false, 1.0),
    (
        -0.002_003_424_264_782_078,
        0.000_762_532_208_608_762_5,
        false,
        false,
        0.998_999_345_025_192_4,
    ),
    (
        0.003_874_244_925_514_067_5,
        0.000_406_125_611_660_040_24,
        false,
        false,
        0.994_776_977_044_112_3,
    ),
    (0.0, 0.0, true, false, 1.0),
    (
        -0.026_025_041_905_665_96,
        -0.006_413_180_917_958_483,
        false,
        false,
        0.967_734_654_646_107_8,
    ),
    (0.0, 0.0, false, true, 0.245_074_383_227_251_8),
];

const EXPECTED_BASIS: [[f64; 3]; 3] = [
    [
        0.936_519_019_120_992_7,
        -0.278_545_055_960_558_7,
        0.212_943_134_720_477_46,
    ],
    [0.0, -0.607_338_685_752_291_4, -0.794_443_025_514_529_5],
    [
        0.350_616_780_580_528_1,
        0.744_011_003_002_381,
        -0.568_784_230_254_968_7,
    ],
];

#[test]
fn matches_typescript_frame_by_frame() {
    let mut pointer = Pointer::new();
    for (i_frame, ((yaw, pitch, roll, click), expected)) in FRAMES.iter().zip(EXPECTED).enumerate()
    {
        let update = pointer.update(&PointerInput {
            rotation: rotation_from_angles(*yaw, *pitch, *roll),
            is_click_pressed: *click,
            delta: FRAME,
            now: FRAME * u32::try_from(i_frame).unwrap(),
        });
        let (x, y, has_recentered, is_aimed_away, facing) = expected;
        assert!(
            (update.position.x - x).abs() < TOLERANCE,
            "frame {i_frame} x"
        );
        assert!(
            (update.position.y - y).abs() < TOLERANCE,
            "frame {i_frame} y"
        );
        assert_eq!(update.has_recentered, has_recentered, "frame {i_frame}");
        assert_eq!(update.is_aimed_away, is_aimed_away, "frame {i_frame}");
        assert!(
            (update.facing - facing).abs() < TOLERANCE,
            "frame {i_frame} facing"
        );
    }
    for (row, expected_row) in pointer.basis().iter().zip(EXPECTED_BASIS) {
        for (value, expected) in row.iter().zip(expected_row) {
            assert!((value - expected).abs() < TOLERANCE);
        }
    }
}

#[test]
fn cycling_candidates_wraps_and_recenters() {
    let mut pointer = Pointer::new();
    pointer.cycle_candidate(CandidateStep::Next);
    assert_eq!(
        pointer.cycle_candidate(CandidateStep::Next).name,
        "measured twist axis"
    );
    assert_eq!(
        pointer.cycle_candidate(CandidateStep::Previous).name,
        "Meta aim ray reversed"
    );
    let update = pointer.update(&PointerInput {
        rotation: rotation_from_angles(0.0, 0.0, 0.0),
        is_click_pressed: false,
        delta: FRAME,
        now: Duration::ZERO,
    });
    assert!(update.has_recentered);
}
