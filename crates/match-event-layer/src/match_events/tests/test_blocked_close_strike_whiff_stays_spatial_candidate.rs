use super::support::*;

#[test]
fn test_blocked_close_strike_whiff_stays_spatial_candidate() {
    let base_frame = 8_670;
    let mut p1 = vec![MeterState::Free; 100];
    let mut p2 = vec![MeterState::Free; 100];
    p1[50..64].fill(MeterState::Recovery);
    p2[50..56].fill(MeterState::Stun);
    p2[56..62].fill(MeterState::Startup);
    p2[62..64].fill(MeterState::Active);
    let block_frame = base_frame + 49;
    let contacts = vec![ContactEvent {
        frame: block_frame,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let punishes = extract_synth_punishes(base_frame, p1, p2, contacts);
    let whiff = punishes
        .iter()
        .find(|punish| punish.side == 2 && punish.outcome == PunishOutcome::WhiffFail)
        .expect("blocked close strike whiff remains for spatial confirmation");
    assert_eq!(whiff.frame, 8_726);
    assert_eq!(whiff.origin, PunishOrigin::BlockedMove);
    assert_eq!(whiff.source_contact_frame, Some(block_frame));
    assert_eq!(whiff.attack_start_frame, Some(8_726));
    assert_eq!(whiff.attack_active_frame, Some(8_732));
    assert_eq!(whiff.reachability, PunishReachability::Unknown);
}
