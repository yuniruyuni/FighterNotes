use crate::digits::UNCOMPUTED_CORRELATION;
use crate::{BrightClass, CellState, RowObs, CELL_COUNT};

#[test]
fn cell_state_text_and_category_contract_is_exhaustive() {
    let cases = [
        (CellState::Counter, "counter", true, false),
        (CellState::PunishCounter, "punish_counter", true, false),
        (CellState::MotionRecovery, "motion_recovery", true, false),
        (CellState::Active, "active", true, false),
        (
            CellState::ProjectileActive,
            "projectile_active",
            true,
            false,
        ),
        (CellState::Parry, "parry", true, false),
        (CellState::Stun, "stun", true, false),
        (CellState::InvFull, "inv_full", true, true),
        (CellState::InvStrike, "inv_strike", true, true),
        (CellState::InvProj, "inv_proj", true, true),
        (CellState::Empty, "empty", false, false),
        (CellState::Other, "other", false, false),
        (CellState::Unknown, "unknown", false, false),
    ];

    for (state, text, is_info, is_stripe) in cases {
        assert_eq!(state.as_str(), text);
        assert_eq!(state.is_info(), is_info, "{text}");
        assert_eq!(state.is_stripe(), is_stripe, "{text}");
        if state != CellState::Unknown {
            assert_eq!(CellState::from_str(text), state);
        }
    }
    assert_eq!(CellState::from_str("not-a-state"), CellState::Unknown);
}

#[test]
fn brightness_text_contract_is_exhaustive() {
    for (class, text) in [
        (BrightClass::Fresh, "fresh"),
        (BrightClass::Low, "low"),
        (BrightClass::None_, "none"),
    ] {
        assert_eq!(class.as_str(), text);
        assert_eq!(BrightClass::from_str(text), class);
    }
    assert_eq!(BrightClass::from_str("not-a-class"), BrightClass::None_);
}

#[test]
fn empty_observation_has_neutral_values_and_missing_sentinels() {
    let row = RowObs::empty();

    assert_eq!(row.v, vec![0.0; CELL_COUNT]);
    assert_eq!(row.wf, vec![0.0; CELL_COUNT]);
    assert_eq!(row.states, vec![CellState::Empty; CELL_COUNT]);
    assert_eq!(row.bright, vec![BrightClass::None_; CELL_COUNT]);
    assert_eq!(row.fresh_edge, -1);
    assert_eq!(row.bgr, vec![[0.0; 3]; CELL_COUNT]);
    assert_eq!(row.stripe, vec![false; CELL_COUNT]);
    assert_eq!(row.cols, None);
    assert_eq!(row.cols_w, 0);
    assert_eq!(row.rescued, vec![false; CELL_COUNT]);
    assert_eq!(row.quality, vec![0.0; CELL_COUNT]);
    assert_eq!(row.digit_corr, None);
    assert_eq!(row.slab_pos, -1);
    assert_eq!(row.slab_state, None);
}

#[test]
fn sparse_digit_correlations_hide_uncomputed_cells() {
    let mut row = RowObs::empty();
    let mut correlations = vec![[UNCOMPUTED_CORRELATION; 10]; CELL_COUNT];
    correlations[7] = [-1.0; 10];
    correlations[7][3] = 0.75;
    row.digit_corr = Some(correlations);

    assert_eq!(row.digit_correlation(7).map(|scores| scores[3]), Some(0.75));
    assert_eq!(row.digit_correlation(8), None);
    assert_eq!(row.digit_correlation(CELL_COUNT), None);
}
