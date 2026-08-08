use super::*;

#[test]
fn overlay_exclusions_remove_changing_meter_region() {
    let mut config = test_config();
    config.excluded_regions = vec![SpatialRect::new(0.2, 0.7, 0.8, 0.9)];
    let mut extractor = SpatialExtractor::new(config);
    let first = blank_frame();
    extractor
        .observe_rgba(400, &first, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    let mut second = first.clone();
    rect(&mut second, 80, 130, 160, 20, [255, 255, 255]);
    let observed = extractor
        .observe_rgba(401, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(observed.projectile_candidates.is_empty());
    assert!(observed.p1.is_none() && observed.p2.is_none());
}
