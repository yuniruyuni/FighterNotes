use super::*;

/// hitstop 中は本体が動かないため、contact ヒントのあるフレームで強く
/// effect 色を帯びた領域はスパークであり、凍結中のトラックへ吸着させない。
/// 除外帯の影響で mid-screen のトラック anchor は地上帯より上にあり、
/// leaves-ground ゲートが効かないので、このゲートが唯一の防壁になる。
#[test]
fn contact_hint_keeps_effect_region_off_actor_tracks() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    // 胴体しか見えない実映像を模して、anchor を地上帯より上に置く。
    let torso_hints = SpatialHints {
        p1: ActorHint {
            anchor: Some(SpatialPoint::new(0.25, 0.80)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        p2: ActorHint {
            anchor: Some(SpatialPoint::new(0.78, 0.80)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        contact_effect: false,
    };
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, torso_hints)
        .unwrap();

    // 両者は完全凍結(同一画素)。P2 の近くにスパークだけが出る。
    let mut second = first.clone();
    rect(&mut second, 204, 122, 24, 20, [255, 210, 40]);
    let observed = extractor
        .observe_rgba(
            101,
            &second,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: true,
                ..SpatialHints::default()
            },
        )
        .unwrap();

    let p2 = observed.p2.expect("carried P2 track");
    assert!(
        !p2.observed,
        "spark must not capture the frozen track: {p2:?}"
    );
    assert!((p2.anchor.x - 0.78).abs() < 0.02, "{p2:?}");
    let contact = observed.contact.expect("spark contact");
    assert!(
        (contact.center.x - 216.0 / WIDTH as f32).abs() < 0.04,
        "{contact:?}"
    );
}

/// 対照: contact ヒントが無ければ effect ゲートは働かず、同じ発光領域が
/// 通常のモーションとしてトラックへ割り当てられる。ゲートが hitstop の
/// 知識にだけ反応していることを固定する。
#[test]
fn without_the_hint_the_same_region_captures_the_track() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    let torso_hints = SpatialHints {
        p1: ActorHint {
            anchor: Some(SpatialPoint::new(0.25, 0.80)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        p2: ActorHint {
            anchor: Some(SpatialPoint::new(0.78, 0.80)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        contact_effect: false,
    };
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, torso_hints)
        .unwrap();

    let mut second = first.clone();
    rect(&mut second, 204, 122, 24, 20, [255, 210, 40]);
    let observed = extractor
        .observe_rgba(101, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    let p2 = observed.p2.expect("P2 track");
    assert!(p2.observed, "{p2:?}");
    assert!((p2.anchor.x - 216.0 / WIDTH as f32).abs() < 0.03, "{p2:?}");
    assert!(observed.contact.is_none());
}
