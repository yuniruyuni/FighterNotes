use super::super::*;
use meter_tracker::{TimelineEntry, TimelineSegment};

macro_rules! extract_test_threats {
    (
        $features:expr,
        $timelines:expr,
        $meter_state:expr,
        $segments:expr,
        $jumps:expr,
        $contacts:expr,
        $damage:expr,
        $rounds:expr,
        $characters:expr $(,)?) => {
        extract_threats(ThreatInputs {
            features: $features,
            timelines: $timelines,
            meter_state: $meter_state,
            segments: $segments,
            jumps: $jumps,
            contacts: $contacts,
            damage: $damage,
            rounds: $rounds,
            characters: $characters,
        })
    };
}

pub(crate) use extract_test_threats;

pub fn timeline(side: &str, runs: &[(u32, u32, &str)]) -> MeterTimeline {
    MeterTimeline {
        side: side.to_string(),
        segments: runs
            .iter()
            .enumerate()
            .map(|(segment_id, &(start, end, state))| TimelineSegment {
                segment_id: segment_id as i32,
                entries: (start..=end)
                    .map(|frame| TimelineEntry {
                        game_frame: frame as i64,
                        state: state.to_string(),
                        video_frame_first: frame as i64,
                        video_frame_last: frame as i64,
                        confidence: 1.0,
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub fn feature(frame_index: u32) -> FrameFeatures {
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
        left_super_uncertain: true,
        right_super_uncertain: true,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: 1.0,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

pub fn round() -> RoundInfo {
    RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 399,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }
}

pub fn teleport_segment(frame: u32) -> InputSegment {
    InputSegment {
        start_frame: frame,
        end_frame: frame + 2,
        dir: "L".to_string(),
        badges: vec!["弱P".to_string(), "中P".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}
