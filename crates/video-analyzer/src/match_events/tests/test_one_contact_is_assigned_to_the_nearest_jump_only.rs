use super::support::*;

#[test]
fn test_one_contact_is_assigned_to_the_nearest_jump_only() {
    let mut fs = Vec::new();
    for i in 0..205u32 {
        fs.push(feat(i, 1.0, 1.0));
    }
    for i in 205..225u32 {
        fs.push(feat(i, 1.0, 1.0 - 0.005 * (i - 204) as f32));
    }
    for i in 225..500u32 {
        fs.push(feat(i, 1.0, 0.9));
    }
    let inputs = up_inputs(fs.len(), &[(160, 164), (180, 184)]);
    let left = synth_timeline(vec![(100, "active", 200, 209)]);
    let right = synth_timeline(
        [
            synth_run(0, "motion_recovery", 160, 174),
            synth_run(20, "motion_recovery", 180, 194),
            vec![(40, "stun", 200, 209)],
        ]
        .concat(),
    );

    let ev = build_match_events(&fs, &[], &inputs, Some((&left, &right)), "p1");
    assert!(
        ev.contacts
            .iter()
            .any(|contact| contact.frame == 200 && contact.hit),
        "合成したヒットコンタクトを検出する: contacts={:?} damage={:?}",
        ev.contacts,
        ev.damage
    );
    let got_hit: Vec<_> = ev
        .jumps
        .iter()
        .filter(|jump| jump.outcome == JumpOutcome::GotHit)
        .collect();
    assert_eq!(got_hit.len(), 1, "同じ接触を二重帰属しない: {:?}", ev.jumps);
    assert_eq!(
        (got_hit[0].frame, got_hit[0].contact_frame),
        (180, Some(200))
    );
    assert_eq!(
        ev.jumps
            .iter()
            .filter(|jump| jump.contact_frame == Some(200))
            .count(),
        1
    );
}
