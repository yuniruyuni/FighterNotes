use super::support::*;

#[test]
fn test_throw_connected() {
    // P1 がフレーム 200 で投げ入力 → 215 から P2 が -0.12
    let mut fs = Vec::new();
    for i in 0..215u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 215..235u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.006 * (i - 214) as f32));
    }
    for i in 235..600u32 {
        fs.push(feat(i, 1.0, 0.88));
    }
    let n = fs.len();
    let mut p1in: Vec<TrackedInput> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    for (k, item) in p1in.iter_mut().enumerate().take(210).skip(200) {
        *item = tracked((k - 199) as u32, InputDir::Neutral, vec![], false, true);
    }
    for (k, item) in p1in.iter_mut().enumerate().skip(210) {
        *item = tracked((k - 209) as u32, InputDir::Neutral, vec![], false, false);
    }
    let ev = build_match_events(&fs, &p1in, &[], None, "p1");
    let th: Vec<_> = ev.throws.iter().filter(|t| t.thrower == 1).collect();
    assert_eq!(th.len(), 1);
    assert!(th[0].connected);
}
