use super::super::*;
use super::support::empty_events;
use crate::{
    attack_info::AttackAttribute,
    match_events::{
        AttackDamageConsistency, ContactEvent, DamageAttackEvidence, DamageEvent, EventConfidence,
        GuardBreakEvent, PunishChance, PunishOrigin, PunishOutcome, PunishReachability,
        SuperArtAttackEvidence, SuperArtContext, SuperArtEvent, SuperArtOutcome,
    },
};
use advice_stats::stats::build_tactic_stats;

fn damage(frame: u32, victim: u8, drop: f32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame: frame,
        pre_freeze_frame: frame,
        end_frame: frame + 20,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn attack(
    frame: u32,
    victim: u8,
    combo_damage: u32,
    attribute: AttackAttribute,
) -> DamageAttackEvidence {
    DamageAttackEvidence {
        victim,
        attacker: 3 - victim,
        damage_start_frame: frame,
        sequence_start_frame: frame,
        sequence_end_frame: frame + 20,
        combo_damage,
        sequence_count: 1,
        final_scaling_percent: 50,
        starter_attribute: Some(attribute),
        final_attribute: attribute,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    }
}

#[test]
fn big_hit_uses_exact_damage_attribute_and_scaling() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.30));
    events
        .attack_evidence
        .damage
        .push(attack(100, 1, 3030, AttackAttribute::Middle));

    let card = detect_big_hits(&events, 1, &[]).expect("big hit");

    assert!(card.evidence[0].label.contains("3030ダメージ"));
    assert!(card.evidence[0].label.contains("中段始動"));
    assert!(card.evidence[0].label.contains("最終50%補正"));
}

#[test]
fn guard_break_excludes_a_centrally_confirmed_throw() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.10));
    events.guard_breaks.push(GuardBreakEvent {
        side: 1,
        frame: 100,
        drop: 0.10,
        guard_dir: "DR".to_string(),
        broke_to: "R".to_string(),
        round_no: 1,
    });
    events
        .attack_evidence
        .damage
        .push(attack(100, 1, 1000, AttackAttribute::Throw));

    assert!(detect_guard_break(&events, 1).is_none());

    events.attack_evidence.damage[0].starter_attribute = Some(AttackAttribute::Lower);
    let card = detect_guard_break(&events, 1).expect("low guard break");
    assert!(card.description.contains("下段"));
    assert!(card.evidence[0].label.contains("1000ダメージ"));
}

#[test]
fn low_punish_return_shows_the_game_reported_damage() {
    let mut events = empty_events();
    events.damage.push(damage(100, 2, 0.05));
    events.contacts.push(ContactEvent {
        frame: 100,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    events.punishes.push(PunishChance {
        frame: 100,
        side: 1,
        advantage: 8,
        outcome: PunishOutcome::Success,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 95,
        recovery_end_frame: 108,
        source_contact_frame: Some(94),
        attack_start_frame: Some(100),
        attack_active_frame: Some(100),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.05,
        pressed: "弱".to_string(),
        round_no: 1,
    });
    events
        .attack_evidence
        .damage
        .push(attack(100, 2, 500, AttackAttribute::Upper));

    let card = detect_low_conversion(&events, 1).expect("low conversion");

    assert!(card.description.contains("500"));
    assert!(card.evidence[0].label.contains("500ダメージ"));
    assert!(card.evidence[0].label.contains("最終50%補正"));
}

#[test]
fn low_scaling_super_reports_marginal_damage_and_updates_stats() {
    let mut events = empty_events();
    events.super_arts.push(SuperArtEvent {
        side: 1,
        frame: 200,
        gauge_drop_frame: 210,
        level: 3,
        critical_art: false,
        gauge_before: 3.0,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome: SuperArtOutcome::Hit,
        contact_frame: Some(220),
        damage: 0.0,
        ko: false,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.damage.push(damage(220, 2, 0.3));
    events
        .attack_evidence
        .damage
        .push(attack(220, 2, 3000, AttackAttribute::Upper));
    events
        .attack_evidence
        .super_arts
        .push(SuperArtAttackEvidence {
            side: 1,
            super_frame: 200,
            combo_damage: 3000,
            marginal_damage: Some(1000),
            entry_scaling_percent: Some(40),
            final_scaling_percent: 40,
            confidence: EventConfidence::High,
        });

    let card = detect_low_scaling_super(&events, 1).expect("low scaling super");
    assert!(card.description.contains("合計 1000"));
    assert!(card.evidence[0].label.contains("投入時40%補正"));
    assert!(card.evidence[0].label.contains("SA以降+1000"));

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.super_damage_samples, 1);
    assert_eq!(stats.super_reported_combo_damage, 3000);
    assert_eq!(stats.super_reported_marginal_damage, 1000);
    assert_eq!(stats.super_low_scaling_uses, 1);

    events.attack_evidence.damage[0].hp_consistency = AttackDamageConsistency::Mismatch;
    assert!(detect_low_scaling_super(&events, 1).is_none());
    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.super_damage_samples, 0);
    assert_eq!(stats.super_reported_combo_damage, 0);
    assert_eq!(stats.super_low_scaling_uses, 0);
}

#[test]
fn opposite_side_attack_evidence_does_not_validate_super_damage() {
    let mut events = empty_events();
    events.super_arts.push(SuperArtEvent {
        side: 1,
        frame: 200,
        gauge_drop_frame: 210,
        level: 1,
        critical_art: false,
        gauge_before: 1.0,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome: SuperArtOutcome::Hit,
        contact_frame: Some(220),
        damage: 0.1,
        ko: false,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events
        .attack_evidence
        .super_arts
        .push(SuperArtAttackEvidence {
            side: 1,
            super_frame: 200,
            combo_damage: 1000,
            marginal_damage: Some(500),
            entry_scaling_percent: Some(40),
            final_scaling_percent: 40,
            confidence: EventConfidence::High,
        });
    events.damage.push(damage(220, 1, 0.1));
    events
        .attack_evidence
        .damage
        .push(attack(220, 1, 1000, AttackAttribute::Upper));

    assert!(detect_low_scaling_super(&events, 1).is_none());
    assert_eq!(
        build_tactic_stats(&[], &events, 1, 2).super_damage_samples,
        0
    );
}
