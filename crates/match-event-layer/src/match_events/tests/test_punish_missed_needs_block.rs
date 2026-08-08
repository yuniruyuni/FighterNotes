use super::support::*;

#[test]
fn test_punish_missed_needs_block() {
    // own_side="p1"、own = P1。相手 = P2。
    let mut fs = Vec::new();
    for i in 0..600u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    // ラウンドが妥当性フィルタで消えないよう、無関係な被弾を 1 つ入れる
    for i in 300..320u32 {
        fs[i as usize] = feat(i, 1.0 - 0.01 * (i - 299) as f32, 1.0);
    }
    for i in 320..600u32 {
        fs[i as usize] = feat(i, 0.8, 1.0);
    }
    // 相手 P2 の後隙 f200-230。直前 f188 に P2 の攻撃を P1 がガード（block）。
    // P1 は何もしない → Missed
    let p2 = synth_timeline(
        [
            vec![(40, "active", 188, 195)],
            synth_run(50, "punish_counter", 200, 230),
        ]
        .concat(),
    ); // P1 がガード → 後隙 31F（1 gf = 1 エントリの実形状）
    let p1 = synth_timeline(
        [
            vec![(40, "stun", 188, 195)],
            synth_run(48, "empty", 196, 230),
        ]
        .concat(),
    );
    let ev = build_match_events(&fs, &[], &[], Some((&p1, &p2)), "p1");
    let missed: Vec<_> = ev
        .punishes
        .iter()
        .filter(|p| p.side == 1 && p.outcome == PunishOutcome::Missed)
        .collect();
    assert_eq!(missed.len(), 1, "確反見逃し: {:?}", ev.punishes);
    assert_eq!(missed[0].reachability, PunishReachability::Unknown);

    // block が無ければ Missed にしない（距離不明のため）
    let p2b = synth_timeline(vec![(50, "punish_counter", 200, 230)]);
    let p1b = synth_timeline(vec![]);
    let ev = build_match_events(&fs, &[], &[], Some((&p1b, &p2b)), "p1");
    assert!(
        ev.punishes
            .iter()
            .all(|p| p.outcome != PunishOutcome::Missed),
        "block 無しでは見逃しにしない: {:?}",
        ev.punishes
    );
}
