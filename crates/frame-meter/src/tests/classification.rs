use crate::classification::fresh_v_min_for;
use crate::palette::PaletteName;
use crate::{
    classify_cell_pair, classify_cell_raw, BrightClass, CellState, EMPTY_V_MAX, STRIPE_WF_MIN,
};

use super::{assert_close, bgr_from_hsv};

#[test]
fn fresh_thresholds_exist_only_for_colored_states() {
    let expected = [
        (CellState::Counter, 172.0),
        (CellState::PunishCounter, 145.0),
        (CellState::MotionRecovery, 216.0),
        (CellState::Active, 157.0),
        (CellState::ProjectileActive, 160.0),
        (CellState::Stun, 219.0),
        (CellState::Parry, 95.0),
    ];
    for (state, threshold) in expected {
        assert_close(fresh_v_min_for(&state).unwrap(), threshold);
    }
    for state in [
        CellState::InvFull,
        CellState::InvStrike,
        CellState::InvProj,
        CellState::Empty,
        CellState::Other,
        CellState::Unknown,
    ] {
        assert_eq!(fresh_v_min_for(&state), None);
    }
}

#[test]
fn pair_distance_limit_is_inclusive_for_each_sample() {
    let boundary = [187.0, 17.0, 65.0];
    let black = PaletteName::Black.color();

    assert_eq!(
        classify_cell_pair(boundary, black),
        (CellState::Parry, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(black, boundary),
        (CellState::Parry, BrightClass::Fresh)
    );
}

#[test]
fn pair_stripe_rules_require_both_expected_palette_roles() {
    let white = PaletteName::White.color();
    let gray = PaletteName::Gray.color();
    let active = PaletteName::Active.color();
    let counter = PaletteName::Counter.color();

    assert_eq!(
        classify_cell_pair(white, active),
        (CellState::Active, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(counter, gray),
        (CellState::Counter, BrightClass::Fresh)
    );
}

#[test]
fn pair_same_family_is_fresh_only_when_both_samples_are_fresh() {
    assert_eq!(
        classify_cell_pair(
            PaletteName::Counter.color(),
            PaletteName::CounterDim.color()
        ),
        (CellState::Counter, BrightClass::Low)
    );
}

#[test]
fn pair_stripe_brightness_requires_both_samples_to_be_full_palette_colors() {
    for (second, expected) in [
        (PaletteName::GrayDim, CellState::InvFull),
        (PaletteName::StripePinkDim, CellState::InvStrike),
        (PaletteName::StripeOrangeDim, CellState::InvProj),
    ] {
        assert_eq!(
            classify_cell_pair(PaletteName::White.color(), second.color()),
            (expected, BrightClass::Low)
        );
    }
}

#[test]
fn pair_empty_requires_both_samples_to_be_emptyish() {
    assert_eq!(
        classify_cell_pair(PaletteName::Black.color(), PaletteName::Gray.color()),
        (CellState::Other, BrightClass::None_)
    );
    assert_eq!(
        classify_cell_pair(PaletteName::Gray.color(), PaletteName::Black.color()),
        (CellState::Other, BrightClass::None_)
    );
}

#[test]
fn raw_classification_preserves_strict_boundaries() {
    assert_eq!(
        classify_cell_raw([0.0; 3], STRIPE_WF_MIN, None),
        CellState::Empty
    );
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(50.0, 60.0, 100.0), 0.0, Some(true)),
        CellState::InvProj
    );
    assert_eq!(
        classify_cell_raw([EMPTY_V_MAX; 3], 0.0, Some(false)),
        CellState::Other
    );
}

#[test]
fn raw_counter_accepts_either_low_hue_or_high_saturation_evidence() {
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(80.0, 100.0, 180.0), 0.0, Some(false)),
        CellState::Counter
    );
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(90.0, 210.0, 180.0), 0.0, Some(false)),
        CellState::Counter
    );
}
