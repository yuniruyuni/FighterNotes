use super::support::*;

#[test]
fn input_stats_and_coverage_use_only_validated_rounds() {
    use crate::match_events::InputSegment;
    let mut ev = empty_events();
    ev.rounds[0].end_frame = 99;
    let press = |frame: u32| InputSegment {
        start_frame: frame,
        end_frame: frame + 4,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    ev.segments[0] = vec![press(10), press(200)];
    let features: Vec<_> = (0..300u32)
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
            left_hp_raw: 1.0,
            right_hp_raw: 1.0,
            left_hp_raw_quality: 0.0,
            right_hp_raw_quality: 0.0,
        })
        .collect();

    let report = build_report(&features, &ev, "p1", None);
    let stats = report.input_stats.expect("ラウンド内入力は集計される");
    assert_eq!(stats.total_inputs, 1, "ラウンド外の入力を混ぜない");
    assert_eq!(stats.button_presses, 1);
    assert_eq!(report.coverage.input_segments, 2);
    assert_eq!(report.coverage.analyzed_input_segments, 1);
    assert_eq!(report.coverage.analyzed_match_frames, 100);
    assert!(report
        .analysis_warnings
        .iter()
        .any(|warning| warning.contains("未検出ラウンド")));
}
