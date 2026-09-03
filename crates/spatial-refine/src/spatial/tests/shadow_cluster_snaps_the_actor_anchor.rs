use super::*;

/// 床帯の影クラスタが近くにあれば、モーション領域から得た anchor.x を
/// 影の重心へ寄せる。マージや部分観測で偏った blob 中心より、接地影の
/// 方が足元の水平位置として信頼できる。
#[test]
fn shadow_cluster_snaps_the_actor_anchor() {
    let mut extractor = SpatialExtractor::new(test_config());
    // 床帯 (y >= 0.87 * 180 = 156.6) に、blob 中心からずらした接地影を描く。
    // セル(4px)に整列させ、重心が正確に予測できるようにする。
    let draw_shadows = |frame: &mut Vec<u8>| {
        rect(frame, 92, 158, 12, 14, [10, 12, 14]);
        rect(frame, 248, 158, 12, 14, [10, 12, 14]);
    };

    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    draw_shadows(&mut first);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut second = blank_frame();
    rect(&mut second, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
    draw_shadows(&mut second);
    let observed = extractor
        .observe_rgba(101, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    // 影セル 23..26 の重心 = 24.5/80 = 0.30625、62..65 の重心 = 63.5/80。
    // blob 中心 (0.272 / 0.756) から離れているので、吸着が無ければ外れる。
    let p1 = observed.p1.expect("P1 track");
    let p2 = observed.p2.expect("P2 track");
    assert!((p1.anchor.x - 0.30625).abs() < 0.012, "{p1:?}");
    assert!((p2.anchor.x - 0.79375).abs() < 0.012, "{p2:?}");
}
