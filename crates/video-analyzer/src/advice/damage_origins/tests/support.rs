use crate::advice::{AdviceCard, AdviceKind, EvidenceClip};
use crate::frame_features::FrameFeatures;
use crate::match_events::{ContactEvent, DamageEvent, EventConfidence, MatchEvents};

pub(super) fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![],
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
        hp: [vec![], vec![]],
    }
}

pub(super) fn damage(frame: u32, victim: u8, drop: f32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame: frame,
        pre_freeze_frame: frame,
        end_frame: frame + 12,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

pub(super) fn contact(frame: u32, projectile: bool) -> ContactEvent {
    ContactEvent {
        frame,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile,
        round_no: 1,
    }
}

pub(super) fn features(count: u32) -> Vec<FrameFeatures> {
    (0..count)
        .map(|frame_index| FrameFeatures {
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
            left_super_uncertain: true,
            right_super_uncertain: true,
            left_ca_ready: false,
            right_ca_ready: false,
            left_hp_raw: 1.0,
            right_hp_raw: 1.0,
            left_hp_raw_quality: 0.0,
            right_hp_raw_quality: 0.0,
        })
        .collect()
}

pub(super) fn advice_card(id: &str, end_frame: Option<u32>) -> AdviceCard {
    AdviceCard {
        id: id.to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::High,
        title: String::new(),
        severity: 0.0,
        hp_lost: None,
        description: String::new(),
        practice: String::new(),
        evidence: vec![EvidenceClip {
            frame: 0,
            end_frame,
            label: String::new(),
        }],
    }
}
