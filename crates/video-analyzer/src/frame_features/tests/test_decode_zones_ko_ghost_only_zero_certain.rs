use super::support::*;

#[test]
fn test_decode_zones_ko_ghost_only_zero_certain() {
    // KO 直後（fill なし、ゴースト残像のみ）→ HP≈0% 確定
    use HpColColor::*;
    let zones = zones_from(&[
        (White, 4),
        (YellowWhite, 1),
        (Ghost, 160),
        (Orange, 6),
        (Dark, 510),
    ]);
    let d = decode_hp_zones(&zones, 681);
    assert!(
        !d.uncertain,
        "KO ゴースト残像のみは uncertain=false であるべき"
    );
    assert!(
        d.fill_ratio < 0.01,
        "KO 時は HP≈0 と読むべき: {}",
        d.fill_ratio
    );
}
