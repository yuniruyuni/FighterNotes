use super::super::{build_damage_breakdown, DAMAGE_ATTRIBUTION_VERSION};
use super::support::{contact, damage, empty_events};
use crate::{
    advice::{DamageApproach, DamageContact, DamageOrigin},
    attack_info::AttackAttribute,
    match_events::{
        AttackDamageConsistency, DamageAttackEvidence, DriveRushEvent, DriveRushOutcome,
        EventConfidence,
    },
};

fn throw_evidence(frame: u32) -> DamageAttackEvidence {
    DamageAttackEvidence {
        victim: 1,
        attacker: 2,
        damage_start_frame: frame,
        sequence_start_frame: frame,
        sequence_end_frame: frame + 12,
        combo_damage: 1200,
        sequence_count: 1,
        final_scaling_percent: 100,
        starter_attribute: Some(AttackAttribute::Throw),
        final_attribute: AttackAttribute::Throw,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    }
}

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

#[test]
fn consistent_central_throw_corrects_a_plain_strike_origin() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.12));
    events.contacts.push(contact(100, false));
    events.attack_evidence.damage.push(throw_evidence(100));

    let breakdown = build_damage_breakdown(&[], &events, 1, None);

    assert_eq!(breakdown.events[0].origin, DamageOrigin::Throw);
    assert_eq!(breakdown.events[0].confidence, EventConfidence::High);
    assert_eq!(breakdown.events[0].approach, None);
    assert_eq!(breakdown.events[0].contact, Some(DamageContact::Throw));
    assert_eq!(
        breakdown.events[0].contact_confidence,
        Some(EventConfidence::High)
    );
}

#[test]
fn central_throw_keeps_a_raw_drive_rush_approach_origin() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.12));
    events.drive_rushes.push(DriveRushEvent {
        side: 2,
        frame: 90,
        raw: true,
        outcome: DriveRushOutcome::Hit,
        contact_frame: Some(100),
        damage: 0.12,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.attack_evidence.damage.push(throw_evidence(100));

    let breakdown = build_damage_breakdown(&[], &events, 1, None);

    assert_eq!(breakdown.events[0].origin, DamageOrigin::RawDriveRush);
    assert_eq!(
        breakdown.events[0].approach,
        Some(DamageApproach::RawDriveRush)
    );
    assert_eq!(breakdown.events[0].contact, Some(DamageContact::Throw));
    assert_eq!(
        breakdown.events[0].contact_confidence,
        Some(EventConfidence::High)
    );
    let json = serde_json::to_value(&breakdown.events[0]).expect("damage event JSON");
    assert_eq!(json["approach"], "raw_drive_rush");
    assert_eq!(json["contact"], "throw");
    assert_eq!(
        breakdown.events[0]
            .attack_evidence
            .as_ref()
            .and_then(|evidence| evidence.starter_attribute),
        Some(AttackAttribute::Throw)
    );
}

#[test]
fn incomplete_or_inconsistent_central_throw_does_not_reclassify() {
    for mutate in [
        |evidence: &mut DamageAttackEvidence| evidence.complete = false,
        |evidence: &mut DamageAttackEvidence| evidence.recovered_from_max = true,
        |evidence: &mut DamageAttackEvidence| {
            evidence.hp_consistency = AttackDamageConsistency::Mismatch
        },
        |evidence: &mut DamageAttackEvidence| evidence.confidence = EventConfidence::Medium,
    ] {
        let mut events = empty_events();
        events.damage.push(damage(100, 1, 0.12));
        events.contacts.push(contact(100, false));
        let mut evidence = throw_evidence(100);
        mutate(&mut evidence);
        events.attack_evidence.damage.push(evidence);

        let breakdown = build_damage_breakdown(&[], &events, 1, None);

        assert_eq!(breakdown.events[0].origin, DamageOrigin::Strike);
    }
}

#[test]
fn raw_drive_rush_contact_is_not_called_throw_without_strict_evidence() {
    for mutate in [
        |evidence: &mut DamageAttackEvidence| evidence.complete = false,
        |evidence: &mut DamageAttackEvidence| evidence.recovered_from_max = true,
        |evidence: &mut DamageAttackEvidence| {
            evidence.hp_consistency = AttackDamageConsistency::Mismatch
        },
        |evidence: &mut DamageAttackEvidence| evidence.confidence = EventConfidence::Medium,
    ] {
        let mut events = empty_events();
        events.damage.push(damage(100, 1, 0.12));
        events.drive_rushes.push(DriveRushEvent {
            side: 2,
            frame: 90,
            raw: true,
            outcome: DriveRushOutcome::Hit,
            contact_frame: Some(100),
            damage: 0.12,
            confidence: EventConfidence::High,
            round_no: 1,
        });
        let mut evidence = throw_evidence(100);
        mutate(&mut evidence);
        events.attack_evidence.damage.push(evidence);

        let event = build_damage_breakdown(&[], &events, 1, None)
            .events
            .remove(0);

        assert_eq!(event.origin, DamageOrigin::RawDriveRush);
        assert_eq!(event.approach, Some(DamageApproach::RawDriveRush));
        assert_ne!(event.contact, Some(DamageContact::Throw));
    }
}
