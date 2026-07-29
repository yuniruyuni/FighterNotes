use super::support::*;

#[test]
fn confirmed_jump_contact_with_obscured_hp_is_sent_to_spatial_refinement() {
    let mut features = Vec::new();
    for frame in 0..200u32 {
        features.push(feat(frame, 1.0, 1.0));
    }
    for frame in 200..220u32 {
        features.push(feat(frame, 1.0 - 0.005 * (frame - 199) as f32, 1.0));
    }
    for frame in 220..400u32 {
        features.push(feat(frame, 0.9, 1.0));
    }
    let p2_inputs = up_inputs(features.len(), &[(100, 104)]);
    let left = synth_timeline(vec![(40, "active", 140, 149)]);
    let right = synth_timeline(
        [
            synth_run(4, "motion_recovery", 104, 139),
            vec![(40, "stun", 140, 149)],
        ]
        .concat(),
    );

    let events = build_match_events(&features, &[], &p2_inputs, Some((&left, &right)), "p1");
    let jump = events
        .jumps
        .iter()
        .find(|jump| jump.side == 2)
        .expect("jump");

    assert!(jump.takeoff_confirmed);
    assert_eq!(jump.contact_frame, Some(140));
    assert_eq!(jump.outcome, JumpOutcome::UnverifiedHit);
    assert!(
        crate::spatial_candidate_windows(&events)
            .iter()
            .any(|window| window.start_frame <= 140 && window.end_frame >= 140),
        "HP だけでは確定できない接触を空間解析へ送る"
    );
}
