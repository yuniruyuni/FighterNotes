use super::*;

/// ガード硬直やダウンで完全静止した本体は、モーション領域を作らなくても
/// 前回観測時と同じ画が同じ場所に残る。これを静止の確認として扱い、
/// max_stale_frames を超えても追跡と信頼度を維持する。
#[test]
fn still_appearance_confirms_a_frozen_actor() {
    let config = test_config();
    let still_confidence = config.still_confidence;
    let max_stale = config.max_stale_frames;
    let mut extractor = SpatialExtractor::new(config);
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut second = blank_frame();
    rect(&mut second, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(101, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    // 以後は完全静止。減衰喪失の上限を大きく超えて維持されることを見る。
    let mut last = None;
    for offset in 0..(max_stale + 10) {
        last = Some(
            extractor
                .observe_rgba(
                    102 + offset,
                    &second,
                    WIDTH,
                    HEIGHT,
                    SpatialHints::default(),
                )
                .unwrap(),
        );
    }
    let observed = last.unwrap();
    let p1 = observed.p1.expect("still-confirmed P1");
    let p2 = observed.p2.expect("still-confirmed P2");
    assert!(!p1.observed && !p2.observed);
    assert_eq!(p1.confidence, still_confidence, "{p1:?}");
    assert_eq!(p2.confidence, still_confidence, "{p2:?}");
    assert!((p1.anchor.x - 0.27).abs() < 0.05, "{p1:?}");
    assert!((p2.anchor.x - 0.76).abs() < 0.05, "{p2:?}");
}
