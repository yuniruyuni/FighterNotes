use super::support::*;

#[test]
fn test_ground_attack_chain_does_not_confirm_takeoff() {
    // 実ゲーム撮影動画のf10692-f10734で観測した事象の同型回帰。
    // 地上技 Active から次の Startup へつながる間に上入力が表示されても、
    // その Startup ランをジャンプの離地証拠にはしない。
    let mut fs = Vec::new();
    for i in 0..132u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 132..152u32 {
        fs.push(feat(i, 1.0 - 0.005 * (i - 131) as f32, 1.0));
    }
    for i in 152..500u32 {
        fs.push(feat(i, 0.9, 1.0));
    }
    let inputs = up_inputs(fs.len(), &[(105, 120)]);
    let left = synth_timeline(vec![(80, "stun", 130, 139)]);
    let right = synth_timeline(
        [
            vec![(49, "active", 99, 99)],
            synth_run(50, "counter", 100, 129),
            vec![(80, "active", 130, 139)],
        ]
        .concat(),
    );

    let events = build_match_events(&fs, &[], &inputs, Some((&left, &right)), "p1");
    let jump = events
        .jumps
        .iter()
        .find(|jump| jump.side == 2 && jump.frame == 105)
        .expect("上入力は空間確認用の候補として保持する");
    assert_eq!(jump.outcome, JumpOutcome::LandedHit);
    assert_eq!(jump.contact_frame, Some(130));
    assert!(
        !jump.takeoff_confirmed,
        "Active→Startup→Active の地上連係を離地確認にしない: {jump:?}"
    );

    let report = crate::advice::build_report(&fs, &events, "p1", Some("CHUN_LI"));
    assert!(report.cards.iter().all(|card| card.id != "anti_air"));
}
