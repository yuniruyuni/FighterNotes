use super::support::*;

#[test]
fn low_punish_return_must_repeat_before_it_is_a_diagnosis() {
    use crate::match_events::ContactEvent;

    let mut ev = empty_events();
    let punish = |frame| PunishChance {
        frame,
        side: 1,
        advantage: 8,
        outcome: PunishOutcome::Success,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: frame - 5,
        recovery_end_frame: frame + 8,
        source_contact_frame: Some(frame - 6),
        attack_start_frame: Some(frame),
        attack_active_frame: Some(frame),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: "弱".to_string(),
        round_no: 1,
    };
    let add_low_return = |events: &mut MatchEvents, frame| {
        events.punishes.push(punish(frame));
        events.contacts.push(ContactEvent {
            frame,
            attacker: 1,
            victim: 2,
            hit: true,
            projectile: false,
            round_no: 1,
        });
        events.damage.push(DamageEvent {
            victim: 2,
            start_frame: frame,
            pre_freeze_frame: frame,
            end_frame: frame + 10,
            hp_before: 1.0,
            hp_after: 0.95,
            drop: 0.05,
            round_no: 1,
        });
    };

    add_low_return(&mut ev, 1000);
    assert_eq!(
        detect_low_conversion(&ev, 1).unwrap().kind,
        AdviceKind::Observation
    );
    add_low_return(&mut ev, 2000);
    assert_eq!(
        detect_low_conversion(&ev, 1).unwrap().kind,
        AdviceKind::Diagnosis
    );
}
