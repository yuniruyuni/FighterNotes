use super::super::build_damage_breakdown;
use super::support::{damage, empty_events};
use crate::match_events::{CompoundThreat, EventConfidence, ProjectileThreat, ThreatOutcome};
use crate::DamageOrigin;

#[test]
fn compound_threat_requires_medium_confidence_and_maps_thresholds() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.1));
    events.compound_threats.push(CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: 50,
        teleport_frame: 70,
        followup_attack_frame: 100,
        followup_contact_frame: Some(100),
        projectile_response: None,
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.1,
        round_no: 1,
        confidence: 0.649,
    });

    let origin = |events: &_| build_damage_breakdown(&[], events, 1, None).events[0].clone();
    assert_eq!(origin(&events).origin, DamageOrigin::Unclassified);
    events.compound_threats[0].confidence = 0.65;
    assert_eq!(origin(&events).confidence, EventConfidence::Medium);
    events.compound_threats[0].confidence = 0.85;
    assert_eq!(origin(&events).confidence, EventConfidence::High);
}

#[test]
fn projectile_threat_needs_a_contact_anchor() {
    let mut events = empty_events();
    events.damage.push(damage(500, 1, 0.06));
    events.projectiles.push(ProjectileThreat {
        owner: 2,
        observed_start_frame: 400,
        observed_end_frame: 420,
        threat_end_frame: 520,
        contact_frame: None,
        round_no: 1,
        confidence: 0.75,
    });

    assert_eq!(
        build_damage_breakdown(&[], &events, 1, None).events[0].origin,
        DamageOrigin::Unclassified
    );
    events.projectiles[0].contact_frame = Some(500);
    let event = &build_damage_breakdown(&[], &events, 1, None).events[0];
    assert_eq!(event.origin, DamageOrigin::Projectile);
    assert_eq!(event.confidence, EventConfidence::Medium);
}
