use super::*;

#[test]
fn reset_allows_a_new_frame_size_after_rejecting_dimension_changes() {
    let mut extractor = SpatialExtractor::new(test_config());
    let first = blank_frame();
    extractor
        .observe_rgba(0, &first, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    let smaller = vec![0; 160 * 90 * 4];
    let error = extractor
        .observe_rgba(1, &smaller, 160, 90, SpatialHints::default())
        .unwrap_err();
    assert!(matches!(error, SpatialError::DimensionsChanged { .. }));

    extractor.reset();
    extractor
        .observe_rgba(2, &smaller, 160, 90, SpatialHints::default())
        .unwrap();
}
