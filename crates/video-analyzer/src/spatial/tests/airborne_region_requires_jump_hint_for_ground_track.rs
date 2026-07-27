use super::*;

#[test]
fn airborne_region_requires_jump_hint_for_ground_track() {
    let mut extractor = SpatialExtractor::new(test_config());
    let first = blank_frame();
    extractor
        .observe_rgba(350, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut airborne = first.clone();
    rect(&mut airborne, 68, 35, 30, 80, [40, 140, 220]);
    let without_hint = extractor
        .observe_rgba(351, &airborne, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(!without_hint.p1.unwrap().observed);

    extractor.reset();
    extractor
        .observe_rgba(350, &first, WIDTH, HEIGHT, hints())
        .unwrap();
    let with_hint = extractor
        .observe_rgba(
            351,
            &airborne,
            WIDTH,
            HEIGHT,
            SpatialHints {
                p1: ActorHint {
                    anchor: None,
                    allow_discontinuity: false,
                    allow_airborne: true,
                },
                p2: ActorHint::default(),
            },
        )
        .unwrap();
    let p1 = with_hint.p1.unwrap();
    assert!(p1.observed && !p1.ground_anchor, "{p1:?}");
}
