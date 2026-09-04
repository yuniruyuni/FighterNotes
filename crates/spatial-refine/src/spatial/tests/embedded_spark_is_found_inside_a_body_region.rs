use super::*;

/// 実際の hitstop では本体も揺れ、スパークが本体のモーション領域へ合体
/// する。スパーク色セルが多く凝集していれば、その重心を衝突位置として
/// 採用する。分散した明色(衣装)は採用しない。
#[test]
fn embedded_spark_is_found_inside_a_body_region() {
    // 本体の大きな移動領域とスパークが 1 領域に合体するフレームを作る。
    // 本体(暗色)が大きく動き、その体表に接する位置へスパークを重ねる。
    let observe = |sparks: &[(u32, u32, u32, u32)]| {
        let mut extractor = SpatialExtractor::new(test_config());
        let mut first = blank_frame();
        rect(&mut first, 60, 60, 60, 90, [70, 80, 90]);
        rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
        extractor
            .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
            .unwrap();
        let mut second = blank_frame();
        rect(&mut second, 72, 60, 60, 90, [70, 80, 90]);
        rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
        for &(x, y, w, h) in sparks {
            rect(&mut second, x, y, w, h, [255, 210, 40]);
        }
        extractor
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
            .unwrap()
    };

    // 本体の右端に重なる 16x16 のスパーク(16 セル、σ 小)。
    let observed = observe(&[(128, 84, 16, 16)]);
    let contact = observed.contact.expect("embedded spark contact");
    assert!(
        (contact.center.x - 136.0 / WIDTH as f32).abs() < 0.03,
        "{contact:?}"
    );
    assert!(
        (contact.center.y - 92.0 / HEIGHT as f32).abs() < 0.04,
        "{contact:?}"
    );

    // スパークが無ければ本体だけでは contact にならない。
    assert!(observe(&[]).contact.is_none());

    // 本体の上下に分かれた明色(衣装の飾り)は合計セル数が足りていても
    // 凝集しておらず、埋め込みスパークとは呼ばない。
    let observed = observe(&[(80, 64, 8, 8), (80, 136, 8, 8)]);
    assert!(observed.contact.is_none(), "{:?}", observed.contact);
}
