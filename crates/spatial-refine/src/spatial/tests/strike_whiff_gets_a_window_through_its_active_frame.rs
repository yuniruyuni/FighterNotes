use super::*;

#[test]
fn strike_whiff_gets_a_window_through_its_active_frame() {
    let mut events = empty_events();
    events.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 7,
        outcome: PunishOutcome::WhiffFail,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 194,
        recovery_end_frame: 207,
        source_contact_frame: Some(193),
        attack_start_frame: Some(200),
        attack_active_frame: Some(205),
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: "弱".to_string(),
        round_no: 1,
    });

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    assert_eq!((windows[0].start_frame, windows[0].end_frame), (157, 213));
}
