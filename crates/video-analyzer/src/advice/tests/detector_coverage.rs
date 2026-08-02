use super::support::*;
use crate::attack_info::AttackAttribute;
use crate::match_events::{
    AttackDamageConsistency, DamageAttackEvidence, DefensiveActionKind, InputEvidence,
    InputSegment, MinusPressEvent, MinusPressOutcome, PunishChance, PunishOrigin, PunishOutcome,
    PunishReachability,
};

fn feature(frame_index: u32) -> FrameFeatures {
    FrameFeatures {
        frame_index,
        fps: 60.0,
        own_hp: 1.0,
        opponent_hp: 1.0,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.1,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_super_value: 0.0,
        right_super_value: 0.0,
        left_super_uncertain: false,
        right_super_uncertain: false,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: 1.0,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

fn neutral_segment(observed_frames: u32) -> InputSegment {
    InputSegment {
        start_frame: 0,
        end_frame: 99,
        dir: "N".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: InputEvidence {
            observed_frames,
            repaired_frames: 100 - observed_frames,
        },
    }
}

fn damage(frame: u32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame: frame,
        pre_freeze_frame: frame,
        end_frame: frame + 1,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }
}

fn attack(frame: u32, hp_consistency: AttackDamageConsistency) -> DamageAttackEvidence {
    DamageAttackEvidence {
        victim: 1,
        attacker: 2,
        damage_start_frame: frame,
        sequence_start_frame: frame,
        sequence_end_frame: frame + 1,
        combo_damage: 1_000,
        sequence_count: 1,
        final_scaling_percent: 100,
        starter_attribute: Some(AttackAttribute::Upper),
        final_attribute: AttackAttribute::Upper,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency,
        sequence_indices: vec![],
    }
}

#[test]
fn report_exposes_detector_specific_numerators_and_attack_states() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 99;
    events.segments = [vec![neutral_segment(80)], vec![neutral_segment(100)]];
    events.input_coverage = crate::match_events::InputCoverage {
        measured: true,
        p1_observed_frames: 80,
        p2_observed_frames: 100,
        p1_repaired_frames: 20,
        p2_repaired_frames: 0,
    };
    events.meter_game_frame = [
        (0..100)
            .map(|frame| if frame < 80 { i64::from(frame) } else { -1 })
            .collect(),
        (0..100).map(i64::from).collect(),
    ];
    events.spatial_coverage = crate::match_events::SpatialCoverage {
        candidate_frames: 20,
        sampled_frames: 10,
        usable_frames: 6,
        p1_observed_frames: 8,
        p2_observed_frames: 7,
    };
    events.damage = vec![damage(10), damage(20), damage(30)];
    events.attack_evidence.damage = vec![
        attack(10, AttackDamageConsistency::Consistent),
        attack(20, AttackDamageConsistency::Mismatch),
        attack(30, AttackDamageConsistency::Unverified),
    ];
    let features: Vec<_> = (0..100)
        .map(|frame| {
            let mut value = feature(frame);
            value.left_hp_raw_quality = f32::from(frame >= 90);
            value.right_hp_raw_quality = f32::from(frame >= 80);
            value.left_drive_uncertain = frame >= 70;
            value.left_super_uncertain = true;
            value.right_super_uncertain = true;
            value
        })
        .collect();

    let report = build_report(&features, &events, "p1", None);
    let coverage = report.coverage;
    assert_eq!(coverage.detector_match_frames, 100);
    assert_eq!(coverage.own_hp_reliable_frames, 90);
    assert_eq!(coverage.opponent_hp_reliable_frames, 80);
    assert_eq!(coverage.own_drive_reliable_frames, 70);
    assert_eq!(coverage.opponent_drive_reliable_frames, 100);
    assert_eq!(coverage.own_super_reliable_frames, 0);
    assert_eq!(coverage.opponent_super_reliable_frames, 0);
    assert!(!coverage.own_super_end_reliable);
    assert!(!coverage.opponent_super_end_reliable);
    assert_eq!(coverage.own_input_observed_frames, 80);
    assert_eq!(coverage.own_input_repaired_frames, 20);
    assert_eq!(coverage.opponent_input_observed_frames, 100);
    assert_eq!(coverage.own_meter_mapped_frames, 80);
    assert_eq!(coverage.opponent_meter_mapped_frames, 100);
    assert_eq!(coverage.spatial_candidate_frames, 20);
    assert_eq!(coverage.spatial_sampled_frames, 10);
    assert_eq!(coverage.spatial_usable_frames, 6);
    assert_eq!(coverage.own_spatial_observed_frames, 8);
    assert_eq!(coverage.opponent_spatial_observed_frames, 7);
    assert_eq!(coverage.attack_damage_events, 3);
    assert_eq!(coverage.attack_damage_linked, 3);
    assert_eq!(coverage.attack_damage_consistent, 1);
    assert_eq!(coverage.attack_damage_mismatched, 1);
    assert_eq!(coverage.attack_damage_unverified, 1);
    assert_eq!(coverage.own_attack_damage_events, 0);
    assert_eq!(coverage.own_attack_damage_usable, 0);
    assert_eq!(coverage.opponent_attack_damage_events, 3);
    assert_eq!(coverage.opponent_attack_damage_usable, 1);
    let availability = coverage.availability.as_ref().expect("new report status");
    assert_eq!(
        availability.own_attack_info,
        EvidenceAvailability::NotApplicable
    );
    assert_eq!(
        availability.opponent_attack_info,
        EvidenceAvailability::Unavailable
    );
    assert_eq!(availability.spatial, EvidenceAvailability::Unavailable);
    assert!(report.input_stats.is_some());
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| warning.contains("使用0回とは断定せず")));
}

#[test]
fn missing_frame_meter_suppresses_dependent_card_and_explains_why() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 99;
    events.segments = [vec![neutral_segment(100)], vec![neutral_segment(100)]];
    events.presses_while_minus.push(MinusPressEvent {
        side: 1,
        frame: 50,
        minus_frames: 4,
        pressed: "弱".to_string(),
        action_kind: DefensiveActionKind::Strike,
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.12,
        confidence: EventConfidence::High,
        source_contact_frame: 40,
        round_no: 1,
    });
    let features: Vec<_> = (0..100).map(feature).collect();

    let report = build_report(&features, &events, "p1", None);

    assert!(report
        .cards
        .iter()
        .all(|card| card.id != "press_while_minus"));
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| warning.contains("フレームメーター") && warning.contains("抑制")));
}

#[test]
fn low_hp_coverage_suppresses_even_generic_big_hit_card() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 99;
    let mut big_hit = damage(10);
    big_hit.hp_after = 0.8;
    big_hit.drop = 0.2;
    events.damage.push(big_hit);
    let features: Vec<_> = (0..100)
        .map(|frame| {
            let mut value = feature(frame);
            value.left_hp_raw_quality = 1.0;
            value
        })
        .collect();

    let report = build_report(&features, &events, "p1", None);

    assert!(report.cards.iter().all(|card| card.id != "big_hits"));
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| warning.contains("HPバー") && warning.contains("抑制")));
}

#[test]
fn missing_attack_info_linkage_is_reported() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 99;
    events.damage = vec![damage(10), damage(20), damage(30)];
    let features: Vec<_> = (0..100).map(feature).collect();

    let report = build_report(&features, &events, "p1", None);

    assert_eq!(report.coverage.attack_damage_events, 3);
    assert_eq!(report.coverage.attack_damage_linked, 0);
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| { warning.contains("中央攻撃表示") && warning.contains("0 / 3") }));
}

#[test]
fn decoded_spatial_frames_without_usable_actor_pair_suppress_reach_card() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 99;
    events.punishes.push(PunishChance {
        frame: 50,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 45,
        recovery_end_frame: 52,
        source_contact_frame: Some(44),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });
    events.meter_game_frame = [
        (0..100).map(i64::from).collect(),
        (0..100).map(i64::from).collect(),
    ];
    events.spatial_coverage = crate::match_events::SpatialCoverage {
        candidate_frames: 20,
        sampled_frames: 20,
        usable_frames: 0,
        p1_observed_frames: 0,
        p2_observed_frames: 0,
    };
    let features: Vec<_> = (0..100).map(feature).collect();

    let report = build_report(&features, &events, "p1", None);

    assert!(report.cards.iter().all(|card| card.id != "punish_missed"));
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| warning.contains("両者の位置") && warning.contains("抑制")));
}

#[test]
fn legacy_coverage_json_defaults_new_detector_fields() {
    let legacy = serde_json::json!({
        "match_frames": 100,
        "analyzed_match_frames": 90,
        "input_segments": 2,
        "analyzed_input_segments": 2
    });

    let coverage: AnalysisCoverage = serde_json::from_value(legacy).expect("legacy coverage");

    assert_eq!(coverage.detector_match_frames, 0);
    assert_eq!(coverage.own_super_reliable_frames, 0);
    assert_eq!(coverage.attack_damage_unverified, 0);
    assert_eq!(coverage.spatial_candidate_frames, 0);
    assert_eq!(coverage.spatial_usable_frames, 0);
    assert!(coverage.availability.is_none());
}

#[test]
fn detector_thresholds_are_inclusive_and_sa_uses_temporal_threshold() {
    assert!(!detector_coverage_is_sufficient(0, 0));
    assert!(!detector_coverage_is_sufficient(59, 100));
    assert!(detector_coverage_is_sufficient(60, 100));
    assert!(!super_coverage_is_sufficient(0, 0));
    assert!(!super_coverage_is_sufficient(19, 100));
    assert!(super_coverage_is_sufficient(20, 100));
    assert!(!spatial_coverage_is_sufficient(0, 0));
    assert!(!spatial_coverage_is_sufficient(19, 100));
    assert!(spatial_coverage_is_sufficient(20, 100));
}

#[test]
fn p2_coverage_maps_sides_and_includes_both_round_endpoints() {
    let mut events = empty_events();
    events.rounds[0].start_frame = 10;
    events.rounds[0].end_frame = 20;
    events.input_coverage = crate::match_events::InputCoverage {
        measured: true,
        p1_observed_frames: 7,
        p2_observed_frames: 3,
        p1_repaired_frames: 4,
        p2_repaired_frames: 8,
    };
    events.spatial_coverage = crate::match_events::SpatialCoverage {
        candidate_frames: 5,
        sampled_frames: 5,
        usable_frames: 5,
        p1_observed_frames: 4,
        p2_observed_frames: 2,
    };
    let mut own_damage = damage(10);
    own_damage.victim = 1;
    let mut opponent_damage = damage(20);
    opponent_damage.victim = 2;
    events.damage = vec![own_damage, opponent_damage];
    let own_attack = attack(10, AttackDamageConsistency::Consistent);
    let mut opponent_attack = attack(20, AttackDamageConsistency::Mismatch);
    opponent_attack.victim = 2;
    opponent_attack.attacker = 1;
    events.attack_evidence.damage = vec![own_attack, opponent_attack];
    events.meter_game_frame = [
        (0..22).map(i64::from).collect(),
        (0..22)
            .map(|frame| {
                if frame == 10 || frame == 20 {
                    frame
                } else {
                    -1
                }
            })
            .collect(),
    ];
    let features: Vec<_> = (9..=21)
        .map(|frame| {
            let mut value = feature(frame);
            value.right_hp_raw_quality = if frame == 10 || frame == 20 { 0.0 } else { 1.0 };
            value.right_drive_uncertain = frame != 10 && frame != 20;
            value
        })
        .collect();

    let report = build_report(&features, &events, "p2", None);
    let coverage = report.coverage;

    assert_eq!(coverage.detector_match_frames, 11);
    assert_eq!(coverage.own_hp_reliable_frames, 2);
    assert_eq!(coverage.opponent_hp_reliable_frames, 11);
    assert_eq!(coverage.own_drive_reliable_frames, 2);
    assert_eq!(coverage.opponent_drive_reliable_frames, 11);
    assert_eq!(coverage.own_input_observed_frames, 3);
    assert_eq!(coverage.opponent_input_observed_frames, 7);
    assert_eq!(coverage.own_meter_mapped_frames, 2);
    assert_eq!(coverage.opponent_meter_mapped_frames, 11);
    assert_eq!(coverage.own_spatial_observed_frames, 2);
    assert_eq!(coverage.opponent_spatial_observed_frames, 4);
    assert_eq!(coverage.own_attack_damage_events, 1);
    assert_eq!(coverage.own_attack_damage_usable, 1);
    assert_eq!(coverage.opponent_attack_damage_events, 1);
    assert_eq!(coverage.opponent_attack_damage_usable, 0);
    let availability = coverage.availability.expect("new report status");
    assert_eq!(availability.own_hp, EvidenceAvailability::Unavailable);
    assert_eq!(availability.opponent_hp, EvidenceAvailability::Available);
    assert_eq!(
        availability.own_attack_info,
        EvidenceAvailability::Available
    );
    assert_eq!(
        availability.opponent_attack_info,
        EvidenceAvailability::Unavailable
    );
}
