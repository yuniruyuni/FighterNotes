use super::*;

/// ガード時の白青スパークも、contact ヒントのあるフレームでは衝突位置に
/// なる。暖色のヒットスパークと同じ経路で、色の判定だけが異なる。
#[test]
fn cold_guard_spark_yields_contact_point() {
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
    rect(&mut second, 140, 60, 16, 16, [245, 250, 255]);
    let observed = extractor
        .observe_rgba(
            101,
            &second,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: true,
                sides_certain: false,
                ..SpatialHints::default()
            },
        )
        .unwrap();

    let contact = observed.contact.expect("cold spark contact");
    assert!(
        (contact.center.x - 148.0 / WIDTH as f32).abs() < 0.03,
        "{contact:?}"
    );

    // 同じ明るさでも warm-gray (b < r) の発光は採用しない。
    let mut extractor = SpatialExtractor::new(test_config());
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();
    let mut gray_flash = blank_frame();
    rect(&mut gray_flash, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut gray_flash, 228, 76, 28, 74, [180, 100, 45]);
    rect(&mut gray_flash, 140, 60, 16, 16, [255, 250, 240]);
    let observed = extractor
        .observe_rgba(
            101,
            &gray_flash,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: true,
                sides_certain: false,
                ..SpatialHints::default()
            },
        )
        .unwrap();
    assert!(observed.contact.is_none(), "{:?}", observed.contact);
}
