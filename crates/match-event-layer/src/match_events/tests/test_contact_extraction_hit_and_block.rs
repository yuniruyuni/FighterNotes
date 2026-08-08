use super::support::*;

#[test]
fn test_contact_extraction_hit_and_block() {
    // f200: P1 active + P2 stun が 10F 停止（ヒット、P2 の HP 減少あり）
    // f400: P1 active + P2 stun が 8F 停止（ガード、HP 減少なし）
    let mut fs = Vec::new();
    for i in 0..205u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 205..225u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.005 * (i - 204) as f32));
    }
    for i in 225..800u32 {
        fs.push(feat(i, 1.0, 0.9));
    }
    let left = synth_timeline(vec![
        (100, "counter", 198, 199),
        (101, "active", 200, 209),
        (102, "punish_counter", 210, 211),
        (200, "active", 400, 407),
        (201, "punish_counter", 408, 409),
    ]);
    let right = synth_timeline(vec![
        (100, "empty", 198, 199),
        (101, "stun", 200, 209),
        (102, "stun", 210, 211),
        (200, "stun", 400, 407),
        (201, "stun", 408, 409),
    ]);
    let ev = build_match_events(&fs, &[], &[], Some((&left, &right)), "p1");
    assert_eq!(ev.contacts.len(), 2, "{:?}", ev.contacts);
    assert_eq!(
        (
            ev.contacts[0].frame,
            ev.contacts[0].attacker,
            ev.contacts[0].hit
        ),
        (200, 1, true)
    );
    assert_eq!(
        (
            ev.contacts[1].frame,
            ev.contacts[1].attacker,
            ev.contacts[1].hit
        ),
        (400, 1, false)
    );
    assert!(!ev.contacts[0].projectile, "打撃接触は弾ではない");
    // 被弾アンカー: HP 減少開始 f205 がコンタクト f200 に寄る
    let d = ev.damage.iter().find(|d| d.victim == 2).unwrap();
    assert_eq!(
        d.start_frame, 200,
        "被弾はコンタクト時刻にアンカーされるべき"
    );
}
