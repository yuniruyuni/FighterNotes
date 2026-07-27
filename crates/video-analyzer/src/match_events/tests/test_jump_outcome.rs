use super::support::*;

#[test]
fn test_jump_outcome() {
    // P2 がフレーム 100 でジャンプ入力 → 130 から P2 被弾（対空された）
    let mut fs = Vec::new();
    for i in 0..100u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 100..130u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 130..150u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.005 * (i - 129) as f32));
    }
    for i in 150..400u32 {
        fs.push(feat(i, 1.0, 0.9));
    }
    let n = fs.len();
    let mut p2in: Vec<TrackedInput> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    for (k, item) in p2in.iter_mut().enumerate().take(130).skip(100) {
        *item = tracked((k - 99) as u32, InputDir::UpLeft, vec![], false, false);
    }
    // ジャンプ後は再びニュートラル（count リセット）
    for (k, item) in p2in.iter_mut().enumerate().skip(130) {
        *item = tracked((k - 129) as u32, InputDir::Neutral, vec![], false, false);
    }
    let p1in: Vec<TrackedInput> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    let ev = build_match_events(&fs, &p1in, &p2in, None, "p1");
    let jumps: Vec<_> = ev.jumps.iter().filter(|j| j.side == 2).collect();
    assert_eq!(
        jumps.len(),
        1,
        "P2 のジャンプが 1 回検出されるべき: {:?}",
        ev.jumps
    );
    assert_eq!(jumps[0].outcome, JumpOutcome::GotHit);
}
