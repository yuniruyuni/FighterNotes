use super::*;

/// contact ヒント付きフレームでは、明るく彩度の高いモーション領域の重心を
/// 衝突位置として返す。hitstop 中は本体が凍結するため、差分に残るのは
/// エフェクトだけという前提を利用する。
#[test]
fn contact_hint_yields_spark_centroid() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    // 本体は小さく動いて追跡を保ち、その間にスパークが出る。
    let mut second = blank_frame();
    rect(&mut second, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
    rect(&mut second, 140, 60, 16, 16, [255, 210, 40]);
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

    let contact = observed.contact.expect("spark contact");
    assert!(
        (contact.center.x - 148.0 / WIDTH as f32).abs() < 0.03,
        "{contact:?}"
    );
    assert!(
        (contact.center.y - 68.0 / HEIGHT as f32).abs() < 0.04,
        "{contact:?}"
    );
    assert!(contact.effect_cells >= 3, "{contact:?}");
    assert!(contact.confidence > 0.5, "{contact:?}");
}
