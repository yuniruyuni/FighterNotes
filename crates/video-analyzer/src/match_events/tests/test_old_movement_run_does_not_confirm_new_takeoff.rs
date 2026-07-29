use super::support::*;

#[test]
fn test_old_movement_run_does_not_confirm_new_takeoff() {
    let mut fs = Vec::new();
    for i in 0..165u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 165..185u32 {
        fs.push(feat(i, 1.0 - 0.005 * (i - 164) as f32, 1.0));
    }
    for i in 185..400u32 {
        fs.push(feat(i, 0.9, 1.0));
    }
    let inputs = up_inputs(fs.len(), &[(100, 104), (150, 154)]);
    let left = synth_timeline(vec![(100, "active", 160, 167)]);
    let right = synth_timeline(
        [
            synth_run(0, "motion_recovery", 100, 159),
            vec![(60, "stun", 160, 167)],
        ]
        .concat(),
    );

    let context =
        crate::context::AnalysisContext::from_characters("p2", Some("LUKE"), Some("CHUN_LI"));
    let ev = build_match_events_with_context(&fs, &[], &inputs, Some((&left, &right)), &context);
    let jumps: Vec<_> = ev.jumps.iter().filter(|jump| jump.side == 2).collect();
    assert_eq!(jumps.len(), 2, "曖昧候補は映像確認用に保持する: {jumps:?}");
    assert_eq!((jumps[0].frame, jumps[0].air_end), (100, 147));
    assert!(jumps[0].takeoff_confirmed);
    assert_eq!(jumps[0].outcome, JumpOutcome::Neutral);
    assert_eq!(jumps[0].contact_frame, None);
    assert_eq!(jumps[1].frame, 150);
    assert!(!jumps[1].takeoff_confirmed);
    assert_eq!(jumps[1].outcome, JumpOutcome::UnverifiedHit);
    assert_eq!(jumps[1].contact_frame, Some(160));
    assert!(jumps
        .iter()
        .all(|jump| jump.air_end <= jump.frame + JUMP_C_HIT_MAX));

    let window = crate::spatial_candidate_windows(&ev)
        .into_iter()
        .find(|window| window.start_frame <= 150 && window.end_frame >= 160)
        .expect("曖昧な離地候補を空間確認へ送る");
    assert!(window
        .airborne_hints
        .iter()
        .any(|hint| hint.side == 2 && hint.end_frame >= 160));

    let report = crate::advice::build_report(&fs, &ev, "p2", Some("LUKE"));
    assert!(report.cards.iter().all(|card| card.id != "own_jumps"));
}

#[test]
fn nearby_takeoff_run_is_preferred_over_a_stale_overlapping_run() {
    let mut fs = Vec::new();
    for i in 0..165u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 165..185u32 {
        fs.push(feat(i, 1.0 - 0.005 * (i - 164) as f32, 1.0));
    }
    for i in 185..300u32 {
        fs.push(feat(i, 0.9, 1.0));
    }
    let inputs = up_inputs(fs.len(), &[(100, 104)]);
    let left = synth_timeline(vec![]);
    let right = synth_timeline(
        [
            synth_run(0, "motion_recovery", 70, 90),
            synth_run(21, "motion_recovery", 104, 142),
        ]
        .concat(),
    );

    let events = build_match_events(&fs, &[], &inputs, Some((&left, &right)), "p1");
    let jumps: Vec<_> = events.jumps.iter().filter(|jump| jump.side == 2).collect();

    assert_eq!(jumps.len(), 1);
    assert_eq!(jumps[0].frame, 100);
    assert!(
        jumps[0].takeoff_confirmed,
        "入力直後の未使用ランを古いランより優先する: {jumps:?}"
    );
}
