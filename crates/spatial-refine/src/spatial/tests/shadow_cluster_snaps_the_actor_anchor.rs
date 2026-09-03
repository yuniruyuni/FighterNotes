use super::*;

/// 床帯の影クラスタが近くにあれば、モーション領域から得た anchor.x を
/// 影の重心へ寄せる。マージや部分観測で偏った blob 中心より、接地影の
/// 方が足元の水平位置として信頼できる。
#[test]
fn shadow_cluster_snaps_the_actor_anchor() {
    let mut extractor = SpatialExtractor::new(test_config());
    // 床帯 (y >= 0.87 * 180 = 156.6) に P1/P2 の接地影を描く。
    let draw_shadows = |frame: &mut Vec<u8>, p1_x: u32, p2_x: u32| {
        rect(frame, p1_x, 158, 40, 14, [10, 12, 14]);
        rect(frame, p2_x, 158, 36, 14, [10, 12, 14]);
    };

    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    draw_shadows(&mut first, 60, 232);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut second = blank_frame();
    rect(&mut second, 72, 78, 30, 72, [40, 140, 220]);
    rect(&mut second, 228, 76, 28, 74, [180, 100, 45]);
    draw_shadows(&mut second, 68, 224);
    let observed = extractor
        .observe_rgba(101, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();

    // 影の重心 = (68 + 40/2) / 320 と (224 + 36/2) / 320 付近。
    let p1 = observed.p1.expect("P1 track");
    let p2 = observed.p2.expect("P2 track");
    assert!((p1.anchor.x - 88.0 / WIDTH as f32).abs() < 0.02, "{p1:?}");
    assert!((p2.anchor.x - 242.0 / WIDTH as f32).abs() < 0.02, "{p2:?}");
}
