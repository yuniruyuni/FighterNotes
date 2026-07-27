use super::support::*;

#[test]
fn test_decode_zones_cap_too_wide_uncertain() {
    // cap 幅 7px（> MAX_CAP_WHITE_WIDTH=6）→ 白遮蔽として uncertain
    use HpColColor::*;
    let zones = zones_from(&[(White, 7), (Fill, 200), (White, 3), (Dark, 471)]);
    let d = decode_hp_zones(&zones, 681);
    assert!(
        d.uncertain,
        "cap 幅 7px は遮蔽として uncertain=true であるべき"
    );
}
