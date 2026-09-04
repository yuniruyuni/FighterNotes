use super::*;

#[test]
fn initial_airborne_hint_assigns_crossed_actor_identity() {
    let mut extractor = SpatialExtractor::new(test_config());
    let first = blank_frame();
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    let mut crossed = first.clone();
    rect(&mut crossed, 64, 90, 30, 65, [40, 140, 220]);
    rect(&mut crossed, 228, 35, 30, 80, [180, 100, 45]);
    let observed = extractor
        .observe_rgba(
            101,
            &crossed,
            WIDTH,
            HEIGHT,
            SpatialHints {
                p1: ActorHint {
                    anchor: None,
                    allow_discontinuity: false,
                    allow_airborne: true,
                },
                p2: ActorHint::default(),
                contact_effect: false,
                sides_certain: false,
            },
        )
        .unwrap();

    let p1 = observed.p1.expect("airborne P1");
    let p2 = observed.p2.expect("grounded P2");
    assert!(
        p1.observed && !p1.ground_anchor && p1.anchor.x > 0.6,
        "{p1:?}"
    );
    assert!(
        p2.observed && p2.ground_anchor && p2.anchor.x < 0.4,
        "{p2:?}"
    );
}
