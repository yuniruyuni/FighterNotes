use super::support::*;

#[test]
fn test_jump_outcome_by_contact_windows() {
    // P2 が f100 でジャンプ入力。コンタクト（P2 被弾ヒット）が f+20（空中）
    // → GotHit。別ケース: コンタクトが f+60（行動可能後）→ Neutral
    let mut fs = Vec::new();
    for i in 0..122u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 122..142u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.005 * (i - 121) as f32));
    }
    for i in 142..700u32 {
        fs.push(feat(i, 1.0, 0.9));
    }
    let n = fs.len();
    let mut p2in: Vec<TrackedInput> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    for (k, item) in p2in.iter_mut().enumerate().take(130).skip(100) {
        *item = tracked((k - 99) as u32, InputDir::UpLeft, vec![], false, false);
    }
    for (k, item) in p2in.iter_mut().enumerate().skip(130) {
        *item = tracked((k - 129) as u32, InputDir::Neutral, vec![], false, false);
    }
    // メーター: P2 のジャンプは移動系ラン（motion）として表示される
    // （離地 +4 から。実測仕様）。f120 で P1 active / P2 stun が 8F 停止
    // （= 入力 f100 の +20F、空中で被弾）
    let left = synth_timeline(vec![(50, "active", 120, 127)]);
    let right = synth_timeline(
        [
            synth_run(10, "motion_recovery", 104, 119),
            vec![(50, "stun", 120, 127)],
        ]
        .concat(),
    );
    let ev = build_match_events(&fs, &[], &p2in, Some((&left, &right)), "p1");
    let j: Vec<_> = ev.jumps.iter().filter(|j| j.side == 2).collect();
    assert_eq!(j.len(), 1);
    assert_eq!(j[0].outcome, JumpOutcome::GotHit, "{:?}", j);

    // コンタクトが f+60 なら着地後 → Neutral（空中 39F の完走ラン）
    let left = synth_timeline(vec![(50, "active", 160, 167)]);
    let right = synth_timeline(
        [
            synth_run(10, "motion_recovery", 104, 142),
            vec![(60, "stun", 160, 167)],
        ]
        .concat(),
    );
    let mut fs2 = Vec::new();
    for i in 0..162u32 {
        fs2.push(feat(i, 1.0, 1.0));
    }
    for i in 162..182u32 {
        fs2.push(feat(i, 1.0, 1.0 - 0.005 * (i - 161) as f32));
    }
    for i in 182..700u32 {
        fs2.push(feat(i, 1.0, 0.9));
    }
    let ev = build_match_events(&fs2, &[], &p2in, Some((&left, &right)), "p1");
    let j: Vec<_> = ev.jumps.iter().filter(|j| j.side == 2).collect();
    assert_eq!(j.len(), 1);
    assert_eq!(
        j[0].outcome,
        JumpOutcome::Neutral,
        "着地後のコンタクトはジャンプ帰属にしない: {:?}",
        j
    );
}
