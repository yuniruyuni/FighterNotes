use super::support::*;

#[test]
fn test_pre_freeze_anchor() {
    // SA 演出: 両者のメーターが f200-280（81vf、FREEZE_MIN_DWELL 超）停止
    // → 演出明け f285 に被弾。pre_freeze_frame はフリーズ開始 f200 になる
    let mut fs = Vec::new();
    for i in 0..600u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 285..305u32 {
        fs[i as usize] = feat(i, 1.0 - 0.02 * (i - 284) as f32, 1.0);
    }
    for i in 305..600u32 {
        fs[i as usize] = feat(i, 0.6, 1.0);
    }
    let p1 = synth_timeline(vec![(10, "stun", 200, 280)]);
    let p2 = synth_timeline(vec![(10, "inv_full", 200, 280)]);
    let ev = build_match_events(&fs, &[], &[], Some((&p1, &p2)), "p1");
    let d = ev.damage.iter().find(|d| d.victim == 1).expect("被弾");
    assert_eq!(d.pre_freeze_frame, 200, "フリーズ開始へ遡る: {d:?}");

    // 対照: フリーズなし（両者 dwell 1 の通常進行）なら start_frame のまま
    let p1n = synth_timeline(synth_run(10, "stun", 270, 284));
    let p2n = synth_timeline(synth_run(10, "active", 270, 284));
    let ev = build_match_events(&fs, &[], &[], Some((&p1n, &p2n)), "p1");
    let d = ev.damage.iter().find(|d| d.victim == 1).expect("被弾");
    assert_eq!(d.pre_freeze_frame, d.start_frame, "フリーズ無しは不変");

    // 対照: 片側だけの長 dwell（単なる長い状態表示）はフリーズではない
    let p1h = synth_timeline(vec![(10, "stun", 200, 280)]);
    let p2h = synth_timeline(synth_run(10, "active", 200, 280));
    let ev = build_match_events(&fs, &[], &[], Some((&p1h, &p2h)), "p1");
    let d = ev.damage.iter().find(|d| d.victim == 1).expect("被弾");
    assert_eq!(
        d.pre_freeze_frame, d.start_frame,
        "片側停止はフリーズではない"
    );
}
