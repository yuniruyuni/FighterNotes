use video_analyzer::FrameFeatures;

fn feature(frame_index: u32, own_hp: f32, opponent_hp: f32) -> FrameFeatures {
    FrameFeatures {
        frame_index,
        fps: 60.0,
        own_hp,
        opponent_hp,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.5,
        right_hp_score: 0.5,
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
        right_hp_raw: opponent_hp,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

pub fn feature_for_p2(frame_index: u32, p2_hp: f32, p1_hp: f32) -> FrameFeatures {
    let mut feature = feature(frame_index, p2_hp, p1_hp);
    feature.left_hp_raw = p1_hp;
    feature.right_hp_raw = p2_hp;
    feature
}

pub fn full_match(frame_count: u32) -> Vec<FrameFeatures> {
    (0..frame_count)
        .map(|frame_index| feature(frame_index, 1.0, 1.0))
        .collect()
}
