use super::support::*;
use crate::match_events::{SuperArtContext, SuperArtEvent, SuperArtOutcome};
use crate::temporal::clean_super_temporal;

#[test]
fn super_art_stats_cover_both_players_and_exact_punished_sa_wording() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 599;
    events.super_arts = vec![
        super_event(1, 100, 2, false, SuperArtOutcome::Blocked, true),
        super_event(2, 300, 3, true, SuperArtOutcome::Hit, false),
    ];

    let stats = build_tactic_stats(&covered_features(599), &events, 1, 2);
    assert_eq!(stats.sa2_used, 1);
    assert_eq!(stats.super_blocked, 1);
    assert_eq!(stats.super_punished, 1);
    assert_eq!(stats.super_combo_uses, 1);
    assert!(stats.super_art_stats_complete);
    assert_eq!(stats.opponent_ca_used, 1);
    assert_eq!(stats.opponent_super_hits, 1);
    assert!(stats.opponent_super_art_stats_complete);

    let card = detect_reversal_punished(&events, 1).expect("punished SA card");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!(card.title.contains("SA/CA"));
    assert!(card.description.contains("SA2"));
    assert!(card.evidence[0].label.contains("SA2"));
}

#[test]
fn super_art_stats_keep_unavailable_distinct_from_zero_uses() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 179;

    let unavailable = build_tactic_stats(&[], &events, 1, 2);
    assert!(!unavailable.super_art_stats_complete);
    assert!(!unavailable.opponent_super_art_stats_complete);
    assert_eq!(unavailable.sa1_used, 0);

    let one_reliable_frame = feature(100, false, true);
    let insufficient = build_tactic_stats(&[one_reliable_frame], &events, 1, 2);
    assert!(!insufficient.super_art_stats_complete);
    assert!(!insufficient.opponent_super_art_stats_complete);

    let complete = build_tactic_stats(&covered_features(179), &events, 1, 2);
    assert!(complete.super_art_stats_complete);
    assert!(complete.opponent_super_art_stats_complete);
    assert_eq!(complete.sa1_used, 0);
}

#[test]
fn one_confirmed_super_does_not_claim_complete_counts_without_coverage() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 179;
    events.super_arts = vec![super_event(1, 100, 1, false, SuperArtOutcome::Hit, false)];

    let stats = build_tactic_stats(&[feature(100, false, false)], &events, 1, 2);
    assert_eq!(stats.sa1_used, 1);
    assert!(!stats.super_art_stats_complete);
    assert!(!stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_requires_every_round() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 179;
    events.rounds.push(crate::match_events::RoundInfo {
        round_no: 2,
        start_frame: 200,
        end_frame: 379,
        winner: Some(1),
        p1_hp_end: 0.5,
        p2_hp_end: 0.0,
    });

    let stats = build_tactic_stats(&covered_features(179), &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(!stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_rejects_a_long_gap_despite_high_overall_observation_rate() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    let mut features = covered_features(399);
    for feature in &mut features[150..229] {
        feature.left_super_uncertain = true;
    }

    let stats = build_tactic_stats(&features, &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_does_not_compress_missing_frame_indexes() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    let mut features = covered_features(399);
    features.retain(|feature| !(150..229).contains(&feature.frame_index));

    let stats = build_tactic_stats(&features, &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(!stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_treats_non_match_frames_as_missing() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    let mut features = covered_features(399);
    for feature in &mut features[150..229] {
        feature.is_match_screen = false;
    }

    let stats = build_tactic_stats(&features, &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(!stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_requires_enough_confirmation_samples_in_every_window() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    let mut features = covered_features(399);
    for feature in &mut features[150..240] {
        feature.left_super_uncertain = true;
    }
    for frame in (150..240).step_by(9) {
        features[frame].left_super_uncertain = false;
    }

    let stats = build_tactic_stats(&features, &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(stats.opponent_super_art_stats_complete);
}

#[test]
fn super_coverage_requires_reliable_round_boundary_frames() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    for boundary in [0, 399] {
        let mut features = covered_features(399);
        features[boundary].left_super_uncertain = true;

        let stats = build_tactic_stats(&features, &events, 1, 2);
        assert!(!stats.super_art_stats_complete);
        assert!(stats.opponent_super_art_stats_complete);
    }
}

#[test]
fn unconfirmed_spend_near_round_end_fails_coverage() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 399;
    let mut features = covered_features(399);
    for feature in &mut features {
        feature.left_super_value = 2.0;
    }
    for feature in &mut features[394..] {
        feature.left_super_value = 0.5;
    }

    clean_super_temporal(&mut features);
    assert!(features[394..]
        .iter()
        .all(|feature| feature.left_super_uncertain));
    let stats = build_tactic_stats(&features, &events, 1, 2);
    assert!(!stats.super_art_stats_complete);
    assert!(stats.opponent_super_art_stats_complete);
}

fn feature(
    frame_index: u32,
    left_super_uncertain: bool,
    right_super_uncertain: bool,
) -> FrameFeatures {
    FrameFeatures {
        frame_index,
        fps: 60.0,
        own_hp: 1.0,
        opponent_hp: 1.0,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 1.0,
        right_hp_score: 1.0,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_super_value: 0.0,
        right_super_value: 0.0,
        left_super_uncertain,
        right_super_uncertain,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: 1.0,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

fn covered_features(end_frame: u32) -> Vec<FrameFeatures> {
    (0..=end_frame)
        .map(|frame| feature(frame, false, false))
        .collect()
}

fn super_event(
    side: u8,
    frame: u32,
    level: u8,
    critical_art: bool,
    outcome: SuperArtOutcome,
    punished: bool,
) -> SuperArtEvent {
    SuperArtEvent {
        side,
        frame,
        gauge_drop_frame: frame + 1,
        level,
        critical_art,
        gauge_before: level as f32,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome,
        contact_frame: Some(frame + 20),
        damage: if outcome == SuperArtOutcome::Hit {
            0.2
        } else {
            0.0
        },
        ko: false,
        punished,
        punished_damage: if punished { 0.18 } else { 0.0 },
        confidence: EventConfidence::High,
        round_no: 1,
    }
}
