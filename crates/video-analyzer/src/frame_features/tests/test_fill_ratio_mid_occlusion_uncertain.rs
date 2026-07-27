use super::support::*;

#[test]
fn test_fill_ratio_mid_occlusion_uncertain() {
    // fill 域中央に 40 列の暗色ブロック（スプライト遮蔽模倣）→ uncertain=true。
    // dark zone width=40 > MAX_DARK_IN_FILL=15 かつ last_fill_zone あり で判定。
    let rgba = make_rgba_p1_bar_with_mid_occlusion(0.6, 430, 40);
    let (_, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(
        uncertain,
        "中央暗色ブロックあり HP バーは uncertain=true であるべき"
    );
}
