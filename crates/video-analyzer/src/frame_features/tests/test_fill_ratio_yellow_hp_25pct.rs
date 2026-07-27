use super::support::*;

#[test]
fn test_fill_ratio_yellow_hp_25pct() {
    // 低 HP 状態（黄色 fill: R=255,G=237,B=0 → H≈28 in OpenCV）を正しく検出すること。
    // is_fill 第2条件 (h 22-35, s>120, v>200) に合致し uncertain=false になるべき。
    let rgba = make_rgba_p1_bar_yellow(0.25);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(!uncertain, "黄色 HP バーは uncertain=false であるべき");
    assert!(
        (fill - 0.25).abs() < 0.03,
        "黄色 HP 25% の fill_ratio が期待範囲外: {fill:.3}"
    );
}
