use crate::frame_features::FrameFeatures;

pub(crate) fn feature(index: u32, own_hp: f32) -> FrameFeatures {
    FrameFeatures {
        frame_index: index,
        fps: 60.0,
        own_hp,
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
        left_super_uncertain: true,
        right_super_uncertain: true,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: own_hp,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

pub(super) fn hp_series(values: &[(f32, f32)]) -> Vec<FrameFeatures> {
    values
        .iter()
        .enumerate()
        .map(|(index, &(own, opponent))| {
            let mut frame = feature(index as u32, own);
            frame.opponent_hp = opponent;
            frame.right_hp_raw = opponent;
            frame
        })
        .collect()
}

pub(super) fn own_hp_series(values: &[f32]) -> Vec<FrameFeatures> {
    values
        .iter()
        .enumerate()
        .map(|(index, &value)| feature(index as u32, value))
        .collect()
}

pub(super) fn drive_series(values: &[(f32, bool, bool)]) -> Vec<FrameFeatures> {
    values
        .iter()
        .enumerate()
        .map(|(index, &(ratio, burnout, uncertain))| {
            let mut frame = feature(index as u32, 1.0);
            frame.left_drive_ratio = ratio;
            frame.left_burnout = burnout;
            frame.left_drive_uncertain = uncertain;
            frame
        })
        .collect()
}

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}
