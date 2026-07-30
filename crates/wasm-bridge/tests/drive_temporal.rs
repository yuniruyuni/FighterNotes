fn feature(
    index: u32,
    ratio: f32,
    burnout: bool,
    uncertain: bool,
) -> video_analyzer::FrameFeatures {
    video_analyzer::FrameFeatures {
        frame_index: index,
        fps: 60.0,
        own_hp: 1.0,
        opponent_hp: 1.0,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.1,
        left_drive_ratio: ratio,
        right_drive_ratio: 1.0,
        left_burnout: burnout,
        right_burnout: false,
        left_drive_uncertain: uncertain,
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

#[test]
fn rejects_short_spike() {
    let mut features: Vec<_> = (0..20)
        .map(|index| feature(index, 0.5, false, false))
        .chain((20..24).map(|index| feature(index, 0.28, false, false)))
        .chain((24..44).map(|index| feature(index, 0.5, false, false)))
        .collect();
    video_analyzer::clean_drive_temporal(&mut features);
    assert!(features[21].left_drive_uncertain);
    assert!((features[21].left_drive_ratio - 0.5).abs() < 1e-6);
    assert!(!features[10].left_drive_uncertain);
    assert!(!features[30].left_drive_uncertain);
}

#[test]
fn keeps_legitimate_drop() {
    let mut features: Vec<_> = (0..20)
        .map(|index| feature(index, 0.8, false, false))
        .chain((20..40).map(|index| feature(index, 0.467, false, false)))
        .collect();
    video_analyzer::clean_drive_temporal(&mut features);
    assert!(!features[10].left_drive_uncertain);
    assert!(!features[30].left_drive_uncertain);
    assert!((features[30].left_drive_ratio - 0.467).abs() < 1e-6);
}

#[test]
fn rejects_occlusion_flicker() {
    let trusted_ratio = 0.268;
    let mut features = Vec::new();
    let mut index = 0;
    append(&mut features, &mut index, 12, trusted_ratio, false, false);
    append(&mut features, &mut index, 20, 0.0, false, true);
    append(&mut features, &mut index, 1, 0.089, false, false);
    append(&mut features, &mut index, 3, 0.0, false, true);
    append(&mut features, &mut index, 1, 0.0, true, false);
    append(&mut features, &mut index, 2, 0.0, false, true);
    append(&mut features, &mut index, 2, 0.092, false, false);
    append(&mut features, &mut index, 10, 0.0, false, true);
    append(&mut features, &mut index, 15, trusted_ratio, false, false);

    video_analyzer::clean_drive_temporal(&mut features);
    let uncertain_end = features.len() - 15;
    for frame in &features[12..uncertain_end] {
        assert!(frame.left_drive_uncertain);
        assert!((frame.left_drive_ratio - trusted_ratio).abs() < 1e-6);
        assert!(!frame.left_burnout);
    }
    assert!(!features.last().unwrap().left_drive_uncertain);
}

#[test]
fn keeps_burnout_entry() {
    let mut features = Vec::new();
    let mut index = 0;
    append(&mut features, &mut index, 12, 0.47, false, false);
    append(&mut features, &mut index, 8, 0.0, false, true);
    append(&mut features, &mut index, 19, 0.0, true, false);
    append(&mut features, &mut index, 8, 0.0, false, true);

    video_analyzer::clean_drive_temporal(&mut features);
    assert!(features[25].left_burnout);
    assert!(!features[25].left_drive_uncertain);
}

#[test]
fn trusts_gradual_drain() {
    let mut features: Vec<_> = (0..30)
        .map(|index| feature(index, 0.9 - index as f32 * 0.01, false, false))
        .collect();
    video_analyzer::clean_drive_temporal(&mut features);
    assert!(!features[15].left_drive_uncertain);
}

fn append(
    features: &mut Vec<video_analyzer::FrameFeatures>,
    next_index: &mut u32,
    count: usize,
    ratio: f32,
    burnout: bool,
    uncertain: bool,
) {
    for _ in 0..count {
        features.push(feature(*next_index, ratio, burnout, uncertain));
        *next_index += 1;
    }
}
