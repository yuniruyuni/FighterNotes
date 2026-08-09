use super::super::build_damage_breakdown;
use super::support::{damage, empty_events};
use crate::match_events::{
    DpReachability, JumpDirection, JumpEvent, JumpOutcome, TeleportContext, TeleportEvent,
    ThreatOutcome,
};
use crate::DamageOrigin;

#[test]
fn teleport_and_opponent_jump_in_are_classified() {
    let mut events = empty_events();
    events.damage = vec![damage(100, 1, 0.1), damage(300, 1, 0.08)];
    events.teleports.push(TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame: 60,
        inv_start_frame: 70,
        inv_end_frame: 80,
        followup_attack_frame: Some(100),
        followup_contact_frame: Some(100),
        airborne: false,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.1,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 0.9,
    });
    events.jumps.push(JumpEvent {
        side: 2,
        frame: 260,
        outcome: JumpOutcome::LandedHit,
        input_dir: "UL".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(300),
        takeoff_confirmed: true,
        air_end: 310,
        round_no: 1,
    });

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert_eq!(breakdown.events[0].origin, DamageOrigin::Teleport);
    assert_eq!(breakdown.events[1].origin, DamageOrigin::OpponentJumpIn);
}
