use super::support::*;

#[test]
fn test_decode_zones_pinch_yellow_with_ghost_trail() {
    // ピンチ黄バー（P2 frame 3440 相当のアンカー正規化形）:
    // cap(5px 淡色膨張) → 黄 fill → YW 境界 → White edge → ゴースト → Dark
    use HpColColor::*;
    let zones = zones_from(&[
        (White, 5),
        (Fill, 160),
        (YellowWhite, 3),
        (White, 3),
        (Ghost, 90),
        (YellowWhite, 3),
        (Dark, 417),
    ]);
    let d = decode_hp_zones(&zones, 681);
    assert!(!d.uncertain, "ピンチ黄バーは uncertain=false であるべき");
    // fill_edge = White edge の遠端 = 5+160+3+3-1 = 170 → (170+1)/681
    assert!(
        (d.fill_ratio - 171.0 / 681.0).abs() < 1e-6,
        "fill_ratio が期待値と不一致: {}",
        d.fill_ratio
    );
    // damage_left = ゴースト帯終端の YW 遠端 = 170+90+3 = 263
    assert!(
        (d.orange_fill - 93.0 / 681.0).abs() < 1e-6,
        "orange_fill が期待値と不一致: {}",
        d.orange_fill
    );
}
