use super::support::*;

#[test]
fn test_punish_excludes_blocked_and_own_sa() {
    // ケース1 型: 相手後隙 f200-230、own が f215 で攻撃 → block（HP不変）。
    //   → 接触したが hit でない = 空振りではない → 機会除外
    // ケース2 型: own が f100-160 で inv_full（SA）→ ガードされて自分が
    //   後隙、相手はガード硬直明け punish_counter。因果 = 自分の攻撃 →除外
    let mut fs = Vec::new();
    for i in 0..800u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    // own = P2。相手 = P1 の後隙 gf ↔ vf を素直に対応させる
    let p1 = synth_timeline(vec![
        (100, "punish_counter", 200, 230),
        (200, "active", 215, 224), // own(P2) の攻撃を P1 がガード → block
    ]);
    let p2 = synth_timeline(vec![
        (100, "counter", 205, 214),
        (101, "active", 215, 224), // 接触するが HP 減少なし = block
    ]);
    let ev = build_match_events(&fs, &[], &[], Some((&p1, &p2)), "p2");
    // block しかない機会は除外される（WhiffFail にしない）
    assert!(
        ev.punishes
            .iter()
            .all(|p| p.side != 2 || p.outcome != PunishOutcome::WhiffFail),
        "ガードされた接触を空振りにしない: {:?}",
        ev.punishes
    );

    // ケース2: own(P2) の inv_full が機会窓に入る → 除外
    let p1b = synth_timeline(vec![(100, "punish_counter", 540, 620)]);
    let p2b = synth_timeline(vec![
        (100, "inv_full", 440, 538),
        (200, "punish_counter", 540, 620),
    ]);
    let ev = build_match_events(&fs, &[], &[], Some((&p1b, &p2b)), "p2");
    assert!(
        ev.punishes.iter().all(|p| p.side != 2),
        "自分の SA 後隙を確反機会にしない: {:?}",
        ev.punishes
    );
}
