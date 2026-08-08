use super::*;

#[test]
fn semantic_windows_merge_and_keep_event_hints() {
    let mut events = empty_events();
    events.teleports = vec![teleport(100), teleport(140)];
    events.compound_threats.push(CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: 60,
        teleport_frame: 100,
        followup_attack_frame: 130,
        followup_contact_frame: Some(130),
        projectile_response: None,
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.1,
        round_no: 1,
        confidence: 0.8,
    });

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    assert_eq!((windows[0].start_frame, windows[0].end_frame), (55, 195));
    assert_eq!(windows[0].teleport_hints.len(), 2);
}
