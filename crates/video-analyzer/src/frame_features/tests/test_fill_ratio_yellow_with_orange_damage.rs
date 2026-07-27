use super::support::*;

#[test]
fn test_fill_ratio_yellow_with_orange_damage() {
    // 低 HP 黄色バー（25%）+ 橙色ダメージ帯の組み合わせ。
    // 黄色 fill（V≈255>200 → is_fill）と橙色 damage（V≈160<200 → Orange）を
    // V 値で正しく分離できること（uncertain=false、fill ≈ 25%）を確認する。
    let rgba = make_rgba_p1_bar_yellow_with_orange(0.25, 350, 100);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(
        !uncertain,
        "黄色 fill + 橙色 damage は uncertain=false であるべき"
    );
    assert!(
        (fill - 0.25).abs() < 0.03,
        "黄色 HP 25% + damage の fill_ratio が期待範囲外: {fill:.3}"
    );
}
