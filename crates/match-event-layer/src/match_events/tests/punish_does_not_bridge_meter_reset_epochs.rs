use super::support::*;

#[test]
fn punish_does_not_bridge_meter_reset_epochs() {
    use MeterState::*;
    let n = 40usize;
    let features: Vec<_> = (0..n).map(|frame| feat(frame as u32, 1.0, 1.0)).collect();
    let own = vec![Free; n];
    let mut opponent = vec![Free; n];
    opponent[10..26].fill(Recovery);
    let mut epochs = vec![0; n];
    epochs[12..].fill(1);
    let game_frames = (0..n as i64).collect::<Vec<_>>();
    let contacts = vec![ContactEvent {
        frame: 9,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let events = super::punishes::extract_punishes(super::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[epochs.clone(), epochs],
        meter_game_frame: &[game_frames.clone(), game_frames],
        contacts: &contacts,
        damage: &[],
        segments: &[vec![], vec![]],
        rounds: &rounds,
    });
    assert!(
        events.is_empty(),
        "リセット前のガードと後の後隙を結ばない: {events:?}"
    );
}
