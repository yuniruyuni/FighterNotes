use super::support::*;

#[test]
fn test_contact_projectile_flag() {
    // f200: P1 の弾を P2 がガード（projectile_active × stun の停止）。
    // 被弾ゼロのラウンドは妥当性フィルタで捨てられるため、離れた位置
    // （f600）に無関係な被弾を置いてラウンドを成立させる
    let mut fs = Vec::new();
    for i in 0..600u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 600..620u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.005 * (i - 599) as f32));
    }
    for i in 620..800u32 {
        fs.push(feat(i, 1.0, 0.9));
    }
    let left = synth_timeline(vec![
        (100, "projectile_active", 200, 207),
        (101, "motion_recovery", 208, 209),
    ]);
    let right = synth_timeline(vec![(100, "stun", 200, 207), (101, "stun", 208, 209)]);
    let ev = build_match_events(&fs, &[], &[], Some((&left, &right)), "p1");
    let c = ev
        .contacts
        .iter()
        .find(|c| c.frame == 200)
        .expect("弾ガード接触");
    assert!(c.projectile, "{c:?}");
    assert!(!c.hit);
}
