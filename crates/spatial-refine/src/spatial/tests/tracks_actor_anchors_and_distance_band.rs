use super::*;

#[test]
fn tracks_actor_anchors_and_distance_band() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut second = blank_frame();
    rect(&mut second, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
    let observed = extractor
        .observe_rgba(101, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    let p1 = observed.p1.expect("P1 motion track");
    let p2 = observed.p2.expect("P2 motion track");
    assert!(p1.observed && p2.observed);
    assert!((p1.anchor.x - 0.26).abs() < 0.08, "{p1:?}");
    assert!((p2.anchor.x - 0.76).abs() < 0.08, "{p2:?}");
    assert_eq!(observed.distance_band, Some(DistanceBand::Far));
    assert_eq!(observed.horizontal_order, Some(HorizontalOrder::P1Left));
}
