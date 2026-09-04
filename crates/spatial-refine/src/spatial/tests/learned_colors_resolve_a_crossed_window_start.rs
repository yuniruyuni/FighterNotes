use super::*;

/// Round 開始直後(側が確定)の window でプレイヤーの色を学習しておくと、
/// めくり後の入れ替わった状態から始まる次の window でも「左=P1」仮定を
/// 学習した色で正し、P1/P2 を取り違えない。
#[test]
fn learned_colors_resolve_a_crossed_window_start() {
    let blue = [40, 140, 220];
    let orange = [180, 100, 45];
    let mut extractor = SpatialExtractor::new(test_config());

    // 確定 window: P1(青)が左、P2(橙)が右で数フレーム動く。
    let certain = SpatialHints {
        sides_certain: true,
        ..hints()
    };
    let mut previous = blank_frame();
    rect(&mut previous, 60, 78, 30, 72, blue);
    rect(&mut previous, 232, 76, 28, 74, orange);
    extractor
        .observe_rgba(100, &previous, WIDTH, HEIGHT, certain)
        .unwrap();
    for step in 1..4u32 {
        let mut frame = blank_frame();
        rect(&mut frame, 60 + step * 4, 78, 30, 72, blue);
        rect(&mut frame, 232 - step * 4, 76, 28, 74, orange);
        extractor
            .observe_rgba(
                100 + step,
                &frame,
                WIDTH,
                HEIGHT,
                SpatialHints {
                    sides_certain: true,
                    ..SpatialHints::default()
                },
            )
            .unwrap();
    }

    // 次の window は側が入れ替わった状態から始まる。
    extractor.reset_window();
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 28, 74, orange);
    rect(&mut first, 236, 76, 30, 72, blue);
    extractor
        .observe_rgba(300, &first, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    let mut second = blank_frame();
    rect(&mut second, 72, 78, 28, 74, orange);
    rect(&mut second, 228, 76, 30, 72, blue);
    let observed = extractor
        .observe_rgba(301, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    let p1 = observed.p1.expect("P1 track");
    let p2 = observed.p2.expect("P2 track");
    assert!(p1.anchor.x > 0.6, "青の P1 は右にいる: {p1:?}");
    assert!(p2.anchor.x < 0.4, "橙の P2 は左にいる: {p2:?}");
    assert_eq!(observed.horizontal_order, Some(HorizontalOrder::P1Right));

    // 完全リセット(別解析)では学習も消え、従来どおり左=P1 に戻る。
    extractor.reset();
    extractor
        .observe_rgba(400, &first, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    let observed = extractor
        .observe_rgba(401, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(observed.p1.expect("P1 track").anchor.x < 0.4);
}
