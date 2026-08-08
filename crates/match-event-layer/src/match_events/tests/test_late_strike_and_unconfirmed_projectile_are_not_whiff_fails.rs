use super::support::*;

#[test]
fn test_late_strike_and_unconfirmed_projectile_are_not_whiff_fails() {
    let base_frame = 100;
    let block = ContactEvent {
        frame: 149,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    };

    let mut p1 = vec![MeterState::Free; 100];
    let mut late_strike = vec![MeterState::Free; 100];
    p1[50..61].fill(MeterState::Recovery);
    late_strike[50..61].fill(MeterState::Startup);
    late_strike[61..64].fill(MeterState::Active);
    let punishes = extract_synth_punishes(base_frame, p1.clone(), late_strike, vec![block.clone()]);
    assert!(punishes
        .iter()
        .all(|punish| punish.outcome != PunishOutcome::WhiffFail));

    let mut projectile = vec![MeterState::Free; 100];
    projectile[50..55].fill(MeterState::Startup);
    projectile[55..58].fill(MeterState::ProjectileActive);
    let punishes = extract_synth_punishes(base_frame, p1, projectile, vec![block]);
    assert!(punishes
        .iter()
        .all(|punish| punish.outcome != PunishOutcome::WhiffFail));
}
