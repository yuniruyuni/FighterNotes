use super::super::build_damage_breakdown;
use super::support::{damage, empty_events};
use crate::match_events::{
    DriveImpactEvent, DriveImpactOutcome, DriveRushEvent, DriveRushOutcome, EventConfidence,
    ThrowActionEvent, ThrowApproach, ThrowOutcome,
};
use crate::DamageOrigin;

#[test]
fn low_confidence_action_candidates_are_ignored() {
    let mut events = empty_events();
    events.damage = vec![
        damage(100, 1, 0.1),
        damage(300, 1, 0.1),
        damage(500, 1, 0.1),
    ];
    events.throw_actions.push(ThrowActionEvent {
        thrower: 2,
        input_frame: 100,
        startup_frame: None,
        active_frame: Some(100),
        outcome: ThrowOutcome::Hit,
        damage: 0.1,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::Low,
        round_no: 1,
    });
    events.drive_impacts.push(DriveImpactEvent {
        side: 2,
        input_frame: 300,
        active_frame: Some(300),
        contact_frame: Some(300),
        outcome: DriveImpactOutcome::Hit,
        damage: 0.1,
        confidence: EventConfidence::Low,
        round_no: 1,
    });
    events.drive_rushes.push(DriveRushEvent {
        side: 2,
        frame: 480,
        raw: true,
        outcome: DriveRushOutcome::Hit,
        contact_frame: Some(500),
        damage: 0.1,
        confidence: EventConfidence::Low,
        round_no: 1,
    });

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert!(breakdown
        .events
        .iter()
        .all(|event| event.origin == DamageOrigin::Unclassified));
}
