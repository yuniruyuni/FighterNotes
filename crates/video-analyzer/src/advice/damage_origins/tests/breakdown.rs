use super::super::{build_damage_breakdown, DAMAGE_ATTRIBUTION_VERSION};
use super::support::{contact, damage, empty_events};
use crate::advice::DamageOrigin;

#[test]
fn empty_breakdown_keeps_current_version_and_zero_totals() {
    let breakdown = build_damage_breakdown(&[], &empty_events(), 1, None);

    assert_eq!(breakdown.attribution_version, DAMAGE_ATTRIBUTION_VERSION);
    assert_eq!(breakdown.total_hp_lost, 0.0);
    assert_eq!(breakdown.classified_hp_lost, 0.0);
    assert!(breakdown.events.is_empty());
}

#[test]
fn breakdown_filters_victim_numbers_sequences_and_preserves_damage() {
    let mut events = empty_events();
    let mut ignored = damage(50, 2, 0.3);
    ignored.pre_freeze_frame = 20;
    let mut classified = damage(100, 1, 0.1);
    classified.pre_freeze_frame = 80;
    let mut unclassified = damage(300, 1, 0.2);
    unclassified.pre_freeze_frame = 350;
    events.damage = vec![ignored, classified, unclassified];
    events.contacts.push(contact(100, false));

    let breakdown = build_damage_breakdown(&[], &events, 1, None);

    assert_eq!(breakdown.events.len(), 2);
    assert_eq!(breakdown.events[0].sequence_no, 1);
    assert_eq!(breakdown.events[1].sequence_no, 2);
    assert_eq!(breakdown.events[0].scene_frame, 80);
    assert_eq!(breakdown.events[1].scene_frame, 300);
    assert_eq!(breakdown.events[0].origin, DamageOrigin::Strike);
    assert_eq!(breakdown.events[1].origin, DamageOrigin::Unclassified);
    assert!((breakdown.total_hp_lost - 0.3).abs() < 1e-6);
    assert!((breakdown.classified_hp_lost - 0.1).abs() < 1e-6);
}
