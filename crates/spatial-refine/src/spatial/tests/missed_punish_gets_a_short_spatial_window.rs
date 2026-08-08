use super::*;

#[test]
fn missed_punish_gets_a_short_spatial_window() {
    let mut events = empty_events();
    events.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    assert_eq!((windows[0].start_frame, windows[0].end_frame), (159, 208));
    assert!(windows[0].teleport_hints.is_empty());
}
