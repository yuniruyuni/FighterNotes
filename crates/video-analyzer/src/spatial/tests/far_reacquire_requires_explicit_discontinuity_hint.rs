use super::*;

#[test]
fn far_reacquire_requires_explicit_discontinuity_hint() {
    let mut extractor = SpatialExtractor::new(test_config());
    let first = blank_frame();
    extractor
        .observe_rgba(300, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut appeared = first.clone();
    rect(&mut appeared, 174, 50, 34, 100, [40, 140, 220]);
    rect(&mut appeared, 236, 76, 28, 74, [180, 100, 45]);
    let without_hint = extractor
        .observe_rgba(301, &appeared, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(!without_hint.p1.unwrap().observed);

    extractor.reset();
    extractor
        .observe_rgba(300, &first, WIDTH, HEIGHT, hints())
        .unwrap();
    let with_hint = extractor
        .observe_rgba(
            301,
            &appeared,
            WIDTH,
            HEIGHT,
            SpatialHints {
                p1: ActorHint {
                    anchor: None,
                    allow_discontinuity: true,
                    allow_airborne: false,
                },
                p2: ActorHint::default(),
            },
        )
        .unwrap();
    let p1 = with_hint.p1.unwrap();
    assert!(p1.observed && p1.discontinuity, "{p1:?}");
    assert!(p1.anchor.x > 0.45);
}
