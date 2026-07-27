use super::support::*;

#[test]
fn test_throw_escape_not_reversal() {
    // 投げ抜け（THROW ESCAPE）: 自分の投げ発生（active）→ 相殺演出の
    // システム無敵（inv_full、実測 42gf）→ 直後に被弾。
    // inv ランの「後」に own Active が続かないので無敵技ではない
    // （実ゲーム撮影動画の観測例: f1738 / f2163）
    let mut fs = Vec::new();
    for i in 0..600u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    // 演出明け f160 から P1 が被弾（投げ抜け後の差し合い負け）
    for i in 160..180u32 {
        fs[i as usize] = feat(i, 1.0 - 0.01 * (i - 159) as f32, 1.0);
    }
    for i in 180..600u32 {
        fs[i as usize] = feat(i, 0.8, 1.0);
    }
    let p1 = synth_timeline(vec![
        (40, "active", 100, 105),   // 自分の投げの発生（inv ランの「前」）
        (41, "inv_full", 106, 150), // 投げ抜け演出のシステム無敵
    ]);
    let p2 = synth_timeline(vec![
        (41, "inv_full", 106, 150), // 相手も同時に無敵（相殺演出）
        (60, "active", 160, 165),   // 演出明けに相手の攻撃がヒット
    ]);
    let ev = build_match_events(&fs, &[], &[], Some((&p1, &p2)), "p1");
    assert!(
        ev.reversals.iter().all(|r| r.side != 1),
        "投げ抜けの無敵を無敵技ぶっぱと誤認しない: {:?}",
        ev.reversals
    );
}
