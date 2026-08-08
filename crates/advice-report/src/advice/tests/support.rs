pub(super) use super::super::*;
pub(super) use crate::match_events::{
    BurnoutPeriod, CompoundThreat, DamageEvent, DefenseResponse, JumpEvent, PunishChance,
    PunishOrigin, RoundInfo, TeleportEvent, ThrowEvent,
};

pub(super) fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 5999,
            winner: Some(2),
            p1_hp_end: 0.0,
            p2_hp_end: 0.5,
        }],
        damage: vec![],
        attack_evidence: Default::default(),
        jumps: vec![],
        throws: vec![],
        throw_actions: vec![],
        drive_impacts: vec![],
        drive_rushes: vec![],
        burnouts: vec![],
        contacts: vec![],
        punishes: vec![],
        reversals: vec![],
        super_arts: vec![],
        guard_breaks: vec![],
        presses_while_minus: vec![],
        minus_situations: vec![],
        advantage_situations: vec![],
        knockdowns: vec![],
        whiffs: vec![],
        projectiles: vec![],
        teleports: vec![],
        compound_threats: vec![],
        meter_state: [vec![], vec![]],
        meter_confidence: [vec![], vec![]],
        meter_game_frame: [vec![], vec![]],
        spatial_coverage: Default::default(),
        input_coverage: Default::default(),
        segments: [vec![], vec![]],
        hp: [vec![1.0; 6000], vec![1.0; 6000]],
    }
}

/// 検出器の契約テストではイベント自体を主題にするため、coverageによる抑制に
/// 必要な各入力を明示的に満たした小さなレポートを組み立てる。
pub(super) fn detector_test_report(events: &MatchEvents, own_side: &str) -> AdviceReport {
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
    build_report(&features, &events, own_side, None)
}

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

pub(super) fn basic_mashing_events() -> MatchEvents {
    use crate::match_events::InputSegment;

    let mut events = empty_events();
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 0.9,
        hp_after: 0.78,
        drop: 0.12,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1200,
        pre_freeze_frame: 1200,
        end_frame: 1220,
        hp_before: 0.78,
        hp_after: 0.66,
        drop: 0.12,
        round_no: 1,
    });
    let press = |start_frame| InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    events.segments[0] = vec![press(990), press(1190)];
    events
}
