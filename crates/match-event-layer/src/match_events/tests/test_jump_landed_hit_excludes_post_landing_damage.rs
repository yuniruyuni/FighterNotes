use super::support::*;

#[test]
fn test_jump_landed_hit_excludes_post_landing_damage() {
    // P1 が f100 でジャンプ → 相手の被弾が f151 から（着地後の地上技）。
    // ジャンプ攻撃のヒット窓（+48F）を過ぎているので LandedHit にしない
    let mut fs = Vec::new();
    for i in 0..151u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 151..171u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.006 * (i - 150) as f32));
    }
    for i in 171..600u32 {
        fs.push(feat(i, 1.0, 0.88));
    }
    let n = fs.len();
    let mut p1in: Vec<TrackedInput> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    for (k, item) in p1in.iter_mut().enumerate().take(140).skip(100) {
        *item = tracked((k - 99) as u32, InputDir::UpRight, vec![], false, false);
    }
    for (k, item) in p1in.iter_mut().enumerate().skip(140) {
        *item = tracked((k - 139) as u32, InputDir::Neutral, vec![], false, false);
    }
    let ev = build_match_events(&fs, &p1in, &[], None, "p1");
    let jumps: Vec<_> = ev.jumps.iter().filter(|j| j.side == 1).collect();
    assert_eq!(jumps.len(), 1);
    assert_eq!(
        jumps[0].outcome,
        JumpOutcome::Neutral,
        "着地後の被弾は LandedHit にしない: {:?}",
        jumps[0]
    );
}
