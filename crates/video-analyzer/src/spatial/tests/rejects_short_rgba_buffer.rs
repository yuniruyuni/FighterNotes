use super::*;

#[test]
fn rejects_short_rgba_buffer() {
    let mut extractor = SpatialExtractor::new(test_config());
    let error = extractor
        .observe_rgba(0, &[0; 4], WIDTH, HEIGHT, SpatialHints::default())
        .unwrap_err();
    assert!(matches!(error, SpatialError::BufferTooSmall { .. }));
}
