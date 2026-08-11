//! SA に結び付いた中央攻撃表示を、厳格な HP 証拠まで含めて集計する。

use super::support::*;
use crate::attack_info::AttackAttribute;
use crate::match_events::{
    AttackDamageConsistency, DamageAttackEvidence, DamageEvent, SuperArtAttackEvidence,
    SuperArtContext, SuperArtEvent, SuperArtOutcome,
};

fn super_art(frame: u32, ko: bool) -> SuperArtEvent {
    SuperArtEvent {
        side: 1,
        frame,
        gauge_drop_frame: frame,
        level: 1,
        critical_art: false,
        gauge_before: 1.0,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome: SuperArtOutcome::Hit,
        contact_frame: Some(frame + 20),
        damage: 0.0,
        ko,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn attach_strict_evidence(
    events: &mut MatchEvents,
    super_frame: u32,
    combo_damage: u32,
    marginal_damage: u32,
    entry_scaling_percent: u32,
) {
    let damage_frame = super_frame + 20;
    events.damage.push(DamageEvent {
        victim: 2,
        start_frame: damage_frame,
        end_frame: damage_frame + 10,
        pre_freeze_frame: super_frame,
        hp_before: 1.0,
        hp_after: 0.8,
        drop: 0.2,
        round_no: 1,
    });
    events.attack_evidence.damage.push(DamageAttackEvidence {
        victim: 2,
        attacker: 1,
        damage_start_frame: damage_frame,
        sequence_start_frame: super_frame,
        sequence_end_frame: damage_frame,
        combo_damage,
        sequence_count: 1,
        final_scaling_percent: entry_scaling_percent,
        starter_attribute: Some(AttackAttribute::Middle),
        final_attribute: AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: Vec::new(),
    });
    events
        .attack_evidence
        .super_arts
        .push(SuperArtAttackEvidence {
            side: 1,
            super_frame,
            combo_damage,
            marginal_damage: Some(marginal_damage),
            entry_scaling_percent: Some(entry_scaling_percent),
            final_scaling_percent: entry_scaling_percent,
            confidence: EventConfidence::High,
        });
}

/// 低 scaling は非 KO の一件だけ。KO の低 scaling と非 KO の通常 scaling
/// も置き、AND・否定・evidence 欠落のどれでも同じ件数にならないようにする。
#[test]
fn strict_attack_evidence_contributes_damage_and_low_scaling_counts() {
    let mut events = empty_events();
    events.super_arts = vec![
        super_art(100, false),
        super_art(500, true),
        super_art(900, false),
        super_art(1_300, false),
        super_art(1_700, true),
    ];
    attach_strict_evidence(&mut events, 100, 2_400, 700, 50);
    attach_strict_evidence(&mut events, 500, 3_000, 800, 40);
    attach_strict_evidence(&mut events, 900, 1_600, 500, 80);
    attach_strict_evidence(&mut events, 1_300, 1_000, 300, 90);
    attach_strict_evidence(&mut events, 1_700, 2_000, 400, 30);

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.super_damage_samples, 5);
    assert_eq!(stats.super_reported_combo_damage, 10_000);
    assert_eq!(stats.super_reported_marginal_damage, 2_700);
    assert_eq!(stats.super_low_scaling_uses, 1);
}
