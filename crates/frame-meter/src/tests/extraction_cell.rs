use crate::color::QuantizedModeScratch;
use crate::extraction::cell;
use crate::extraction::metrics::CellBounds;
use crate::extraction::source::RowPixels;
use crate::{BrightClass, CellState, BLACKISH_PATCH_V, HIGHLIGHT_V_MIN};

fn classify_regions(
    first: &[[u8; 3]],
    second: &[[u8; 3]],
    mean_value: f32,
) -> cell::ClassifiedCell {
    let mut bgr = Vec::from(first);
    bgr.extend_from_slice(second);
    let height = bgr.len();
    let pixels = RowPixels {
        width: 1,
        height,
        trim_y: 0,
        patch_height: height,
        region1_rows: (0..first.len()).chain([height]).collect(),
        region2_rows: (first.len()..height).chain([height]).collect(),
        bgr,
        value: vec![mean_value; height],
        saturation: vec![0.0; height],
    };
    cell::classify(
        &pixels,
        CellBounds { x1: 0, x2: 1 },
        mean_value,
        &mut Vec::new(),
        &mut Vec::new(),
        &mut QuantizedModeScratch::new(),
    )
}

#[test]
fn highlighted_empty_cell_becomes_other_at_inclusive_threshold() {
    let black = [[23, 20, 23]; 3];
    assert_eq!(
        classify_regions(&black, &black, HIGHLIGHT_V_MIN).state,
        CellState::Other
    );
    assert_eq!(
        classify_regions(&black, &black, HIGHLIGHT_V_MIN - 1.0).state,
        CellState::Empty
    );
}

#[test]
fn noisy_mode_is_rescued_only_above_threshold_when_families_match() {
    let mut counter = vec![[255, 0, 255]; 10];
    counter[6..].fill([146, 201, 19]);

    let rescued = classify_regions(&counter, &counter, BLACKISH_PATCH_V + 1.0);
    assert_eq!(rescued.state, CellState::Counter);
    assert_eq!(rescued.bright, BrightClass::Fresh);
    assert!(rescued.rescued);
    assert_eq!(rescued.bgr, [146.0, 201.0, 19.0]);

    let at_threshold = classify_regions(&counter, &counter, BLACKISH_PATCH_V);
    assert_eq!(at_threshold.state, CellState::Other);
    assert!(!at_threshold.rescued);

    let mut active = vec![[255, 0, 255]; 10];
    active[6..].fill([93, 20, 176]);
    assert!(!classify_regions(&counter, &active, BLACKISH_PATCH_V + 1.0).rescued);
}

#[test]
fn empty_mode_can_be_rescued_by_colored_family_pixels() {
    let mut counter = vec![[23, 20, 23]; 10];
    counter[6..].fill([146, 201, 19]);

    let rescued = classify_regions(&counter, &counter, BLACKISH_PATCH_V + 1.0);
    assert_eq!(rescued.state, CellState::Counter);
    assert!(rescued.rescued);
}

#[test]
fn colored_quality_is_distance_to_the_matching_palette() {
    let shifted_counter = [[147, 201, 19]; 3];
    let classified = classify_regions(&shifted_counter, &shifted_counter, 201.0);

    assert_eq!(classified.state, CellState::Counter);
    assert_eq!(classified.quality, 1.0);
}
