use super::super::build_damage_breakdown;
use super::support::{contact, damage, empty_events};
use crate::match_events::{DriveImpactEvent, DriveImpactOutcome, EventConfidence};
use crate::DamageOrigin;

#[test]
fn own_drive_impact_countered_is_attributed_to_drive_impact() {
    let mut events = empty_events();
    events.damage.push(damage(700, 1, 0.16));
    events.drive_impacts.push(DriveImpactEvent {
        side: 1,
        input_frame: 680,
        active_frame: Some(700),
        contact_frame: Some(700),
        outcome: DriveImpactOutcome::Countered,
        damage: 0.16,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert_eq!(breakdown.events[0].origin, DamageOrigin::DriveImpact);
}

#[test]
fn contact_classification_respects_both_window_edges() {
    let mut events = empty_events();
    events.damage = vec![damage(100, 1, 0.05), damage(200, 1, 0.05)];
    events.contacts = vec![contact(75, false), contact(206, false)];

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert_eq!(breakdown.events[0].origin, DamageOrigin::Strike);
    assert_eq!(breakdown.events[1].origin, DamageOrigin::Unclassified);
}
