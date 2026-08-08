use super::support::*;

#[test]
fn test_segments_split_on_input_change() {
    let mut fs = Vec::new();
    for i in 0..30u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    // 10f N → 10f DL → 10f N（count はそれぞれ 1..10）
    let mut p1in = Vec::new();
    for k in 0..10 {
        p1in.push(tracked(k + 1, InputDir::Neutral, vec![], false, false));
    }
    p1in[3].repaired = true;
    for k in 0..10 {
        p1in.push(tracked(k + 1, InputDir::DownLeft, vec![], false, false));
    }
    for k in 0..10 {
        p1in.push(tracked(k + 1, InputDir::Neutral, vec![], false, false));
    }
    let ev = build_match_events(&fs, &p1in, &[], None, "p1");
    assert_eq!(ev.segments[0].len(), 3, "{:?}", ev.segments[0]);
    assert_eq!(ev.segments[0][1].dir, "DL");
    assert_eq!(
        ev.segments[0][0].evidence,
        InputEvidence {
            observed_frames: 9,
            repaired_frames: 1,
        }
    );
}
