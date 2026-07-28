use frame_meter::{
    brightness_class, classify_cell_pair, classify_cell_raw, extract_row_obs,
    extract_row_obs_from_strip, extract_row_obs_from_strip_with_digit_hint, fresh_color_edge,
    BrightClass, CellState, RowObs, BLACKISH_PATCH_V, CELL_COUNT, DIGIT_CHARS, DIGIT_TEMPLATE_H,
    DIGIT_TEMPLATE_W, DIM_V_SCALE, EMPTY_V_MAX, FAMILY_ASSIGN_DIST, HIGHLIGHT_V_MIN, LIT_ROW_V_MIN,
    METER_STRIP_H, METER_STRIP_Y, PAIR_REJECT_DIST, RESCUE_MIN_FRAC, STRIPE_MAX_ROW_XSTD,
    STRIPE_MIN_CONTRAST, STRIPE_MIN_TRANSITIONS, STRIPE_WF_MIN,
};

type FreshColorEdgeFn = fn(&[f32], &[f32], &[CellState], &[BrightClass]) -> i32;
type HintedStripExtractionFn = fn(&[u8], u32, u32, Option<(usize, usize)>) -> (RowObs, RowObs);

#[test]
fn crate_root_keeps_the_detection_api() {
    let _: fn([f32; 3], [f32; 3]) -> (CellState, BrightClass) = classify_cell_pair;
    let _: fn([f32; 3], f32, Option<bool>) -> CellState = classify_cell_raw;
    let _: fn(&CellState, f32, f32) -> BrightClass = brightness_class;
    let _: FreshColorEdgeFn = fresh_color_edge;
    let _: fn(&[u8], u32, u32) -> (RowObs, RowObs) = extract_row_obs;
    let _: fn(&[u8], u32, u32) -> (RowObs, RowObs) = extract_row_obs_from_strip;
    let _: HintedStripExtractionFn = extract_row_obs_from_strip_with_digit_hint;
}

#[test]
fn public_calibration_values_stay_stable() {
    assert_eq!((CELL_COUNT, METER_STRIP_Y, METER_STRIP_H), (80, 796, 78));
    assert_eq!(
        (DIGIT_CHARS, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W),
        ("0123456789", 26, 13)
    );
    assert_eq!(
        (
            HIGHLIGHT_V_MIN,
            BLACKISH_PATCH_V,
            PAIR_REJECT_DIST,
            STRIPE_WF_MIN,
            EMPTY_V_MAX,
            FAMILY_ASSIGN_DIST,
            RESCUE_MIN_FRAC,
        ),
        (90.0, 55.0, 100.0, 0.10, 55.0, 45.0, 0.35),
    );
    assert_eq!(
        (
            STRIPE_MIN_TRANSITIONS,
            STRIPE_MIN_CONTRAST,
            STRIPE_MAX_ROW_XSTD,
            LIT_ROW_V_MIN,
            DIM_V_SCALE,
        ),
        (6, 18.0, 30.0, 60.0, 0.75),
    );
}

#[test]
fn public_models_keep_their_conversions_and_empty_shape() {
    assert_eq!(CellState::from_str("active"), CellState::Active);
    assert_eq!(CellState::Active.as_str(), "active");
    assert!(CellState::InvFull.is_stripe());
    assert!(CellState::Stun.is_info());
    assert_eq!(BrightClass::from_str("fresh"), BrightClass::Fresh);
    assert_eq!(BrightClass::Low.as_str(), "low");

    let row = RowObs::empty();
    assert_eq!(row.v.len(), CELL_COUNT);
    assert_eq!(row.wf.len(), CELL_COUNT);
    assert_eq!(row.states.len(), CELL_COUNT);
    assert_eq!(row.bright.len(), CELL_COUNT);
    assert_eq!(row.bgr.len(), CELL_COUNT);
    assert_eq!(row.rescued.len(), CELL_COUNT);
    assert_eq!(row.quality.len(), CELL_COUNT);
    assert!(row.cols.is_none());
    assert!(row.digit_corr.is_none());
}
