use super::support::*;

#[test]
fn test_naked_teleport_card_requires_confirmed_reach() {
    let mut ev = empty_events();
    ev.teleports.push(TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame: 100,
        inv_start_frame: 110,
        inv_end_frame: 116,
        followup_attack_frame: Some(130),
        followup_contact_frame: Some(130),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.1,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 0.9,
    });

    assert!(detect_teleport_defense(&ev, 1).is_none());
    ev.teleports[0].dp_reachability = DpReachability::Confirmed;
    assert_eq!(
        detect_teleport_defense(&ev, 1).map(|card| card.id),
        Some("teleport_defense".to_string())
    );

    ev.teleports[0].context = TeleportContext::ProjectileCovered;
    assert!(detect_teleport_defense(&ev, 1).is_none());
    ev.teleports[0].context = TeleportContext::DefenderUnavailable;
    assert!(detect_teleport_defense(&ev, 1).is_none());
}
