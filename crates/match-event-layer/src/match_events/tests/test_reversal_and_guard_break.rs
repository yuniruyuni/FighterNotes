use super::support::*;

#[test]
fn test_reversal_and_guard_break() {
    // own_side="p1"（feat の left_hp_raw と hp[0] が整合）。own = P1。
    let mut fs = Vec::new();
    for i in 0..800u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    // (A) 無敵技: P1 が inv_full 100-115 → active 116-125 を P2 がガード（block）
    //     → P1 が f130 で被弾（後隙を狩られた）
    for i in 130..160u32 {
        fs[i as usize] = feat(i, 1.0 - 0.01 * (i - 129) as f32, 1.0);
    }
    for i in 160..425u32 {
        fs[i as usize] = feat(i, 0.7, 1.0);
    }
    // (B) ガード崩れ: P2 の固め（block 350/370）→ f425 で P1 が打撃被弾（ボタン無し）
    for i in 425..445u32 {
        fs[i as usize] = feat(i, 0.7 - 0.01 * (i - 424) as f32, 1.0);
    }
    for i in 445..800u32 {
        fs[i as usize] = feat(i, 0.5, 1.0);
    }
    let p1 = synth_timeline(vec![
        (50, "inv_full", 100, 115),       // 無敵技発生
        (51, "active", 116, 125),         // 攻撃判定（P2 がガード = block）
        (52, "punish_counter", 126, 160), // 後隙（被弾まで同一epoch）
        (60, "stun", 400, 424),           // ブロック硬直（stun + DL 保持 + HP 平坦）
        (61, "stun", 425, 434),           // 被弾（ヒットストップ）
    ]);
    let p2 = synth_timeline(vec![
        (50, "stun", 116, 125),   // P1 の無敵技をガード（P1=active, P2=stun）
        (61, "active", 425, 434), // 打撃ヒット
    ]);
    // P1 の入力: f424 まで DL（ガード方向。P1 は右向きで back=左）を握り、
    // f425 で U（上=ジャンプ）に外れる（＝意図的なガード入力崩れ）
    let n = fs.len();
    let mut p1in: Vec<TrackedInput> = Vec::with_capacity(n);
    for i in 0..n {
        let (c, dir) = if i < 425 {
            ((i + 1) as u32, InputDir::DownLeft)
        } else {
            ((i - 424) as u32, InputDir::Up)
        };
        p1in.push(tracked(c, dir, vec![], false, false));
    }
    let ev = build_match_events(&fs, &p1in, &[], Some((&p1, &p2)), "p1");
    // 無敵技ぶっぱ被弾（ガードされて後隙被弾）
    let rev: Vec<_> = ev.reversals.iter().filter(|r| r.side == 1).collect();
    assert_eq!(rev.len(), 1, "無敵技ぶっぱ被弾: {:?}", ev.reversals);
    assert!(rev[0].blocked, "ガードされたはず");
    // ガード入力崩れ（f425 で DL→U に外れて被弾、直前 block 固め）
    let gb: Vec<_> = ev.guard_breaks.iter().filter(|g| g.side == 1).collect();
    assert_eq!(gb.len(), 1, "ガード入力崩れ: {:?}", ev.guard_breaks);
    assert_eq!(
        (gb[0].guard_dir.as_str(), gb[0].broke_to.as_str()),
        ("DL", "U")
    );
    assert!(gb[0].frame >= 425 && gb[0].frame <= 434);

    // 対照1: ガードを握り続けた（DL のまま）なら崩れではない
    let mut p1hold: Vec<TrackedInput> = Vec::with_capacity(n);
    for i in 0..n {
        p1hold.push(tracked(
            (i + 1) as u32,
            InputDir::DownLeft,
            vec![],
            false,
            false,
        ));
    }
    let ev = build_match_events(&fs, &p1hold, &[], Some((&p1, &p2)), "p1");
    assert!(
        ev.guard_breaks.iter().all(|g| g.side != 1),
        "ガードを握り続けたら崩れにしない: {:?}",
        ev.guard_breaks
    );

    // 対照2: N（ニュートラル）に抜けただけは曖昧なので崩れにしない
    let mut p1n: Vec<TrackedInput> = Vec::with_capacity(n);
    for i in 0..n {
        let dir = if i < 425 {
            InputDir::DownLeft
        } else {
            InputDir::Neutral
        };
        p1n.push(tracked((i + 1) as u32, dir, vec![], false, false));
    }
    let ev = build_match_events(&fs, &p1n, &[], Some((&p1, &p2)), "p1");
    assert!(
        ev.guard_breaks.iter().all(|g| g.side != 1),
        "N 抜けは曖昧なので崩れにしない: {:?}",
        ev.guard_breaks
    );
}
