use super::*;

/// スパーク判定は第一段の contact ヒントが無ければ動かず、追跡中の両者の
/// 間から外れた発光(ステージ演出など)も採用しない。
#[test]
fn contact_spark_requires_hint_and_actor_span() {
    // ヒントなし: 同じスパークがあっても contact は返さない。
    let observed = observe_spark(140, false);
    assert!(observed.is_none(), "{observed:?}");

    // ヒントあり・両者の間: 返す。
    let observed = observe_spark(140, true);
    assert!(observed.is_some());

    // ヒントあり・両者の span 外: ステージ演出とみなして返さない。
    let observed = observe_spark(300, true);
    assert!(observed.is_none(), "{observed:?}");
}

fn observe_spark(spark_x: u32, hint: bool) -> Option<ContactObservation> {
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
    rect(&mut second, spark_x, 60, 12, 12, [255, 210, 40]);
    extractor
        .observe_rgba(
            101,
            &second,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: hint,
                ..SpatialHints::default()
            },
        )
        .unwrap()
        .contact
}
