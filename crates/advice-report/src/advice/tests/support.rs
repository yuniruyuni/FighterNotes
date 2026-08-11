pub(super) use super::super::*;
pub(super) use crate::match_events::{
    BurnoutPeriod, CompoundThreat, DamageEvent, JumpEvent, TeleportEvent, ThrowEvent,
};

pub(super) use match_event_layer::test_support::empty_events;

/// 検出器の契約テストではイベント自体を主題にするため、coverageによる抑制に
/// 必要な各入力を明示的に満たした小さなレポートを組み立てる。
pub(super) fn detector_test_report(events: &MatchEvents, own_side: &str) -> AdviceReport {
    detector_test_report_with_character(events, own_side, None)
}

/// 旧 API のキャラクター引数まで含め、上位レポートへの伝播を検査する版。
pub(super) fn detector_test_report_with_character(
    events: &MatchEvents,
    own_side: &str,
    own_character: Option<&str>,
) -> AdviceReport {
    let mut events = events.clone();
    const FRAMES: u32 = 10;
    events.input_coverage = crate::match_events::InputCoverage {
        measured: true,
        p1_observed_frames: FRAMES,
        p2_observed_frames: FRAMES,
        p1_repaired_frames: 0,
        p2_repaired_frames: 0,
    };
    events.meter_game_frame = [
        (0..FRAMES).map(i64::from).collect(),
        (0..FRAMES).map(i64::from).collect(),
    ];
    events.spatial_coverage = crate::match_events::SpatialCoverage {
        candidate_frames: FRAMES,
        sampled_frames: FRAMES,
        usable_frames: FRAMES,
        p1_observed_frames: FRAMES,
        p2_observed_frames: FRAMES,
    };
    let features: Vec<_> = (0..FRAMES)
        .map(|frame_index| FrameFeatures {
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
        })
        .collect();
    build_report(&features, &events, own_side, own_character)
}

/// 事実確認の指摘は、断定せず利用者の検討を促す文言を必ず含む。
/// ここが崩れると、単発の結果を原因と読ませてしまう。
pub(super) fn assert_invites_user_review(card: &AdviceCard) {
    assert_eq!(
        OBSERVATION_REVIEW_CAVEAT,
        "断定できませんが、検討の対象にしてもよいかもしれません"
    );
    assert!(
        card.description.contains(OBSERVATION_REVIEW_CAVEAT),
        "確認場面が利用者の検討を促していない: {}",
        card.description
    );
}
