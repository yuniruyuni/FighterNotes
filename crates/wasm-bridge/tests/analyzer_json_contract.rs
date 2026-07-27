use serde_json::Value;
use wasm_bridge::Analyzer;

fn assert_object_keys(value: &Value, expected: &[&str]) {
    let mut actual = value
        .as_object()
        .expect("contract value must be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

#[test]
fn analyzer_keeps_buffer_and_json_contracts() {
    let mut analyzer = Analyzer::new("p1");
    assert_eq!(analyzer.hud_buf_len(), 1920 * 70 * 4);
    assert_eq!(analyzer.meter_buf_len(), 1920 * 78 * 4);
    assert_eq!(analyzer.input_buf_len(), 1920 * 36 * 4);
    assert_eq!(analyzer.progress(), 0.0);

    analyzer.analyze_meter_inplace(1920, 1080, 0);
    analyzer.analyze_input_inplace(1920, 1080, 0);
    analyzer.push_hud_features_inplace(1920, 1080, 0);
    assert_eq!(analyzer.progress(), 1.0);

    let features: Value = serde_json::from_str(&analyzer.get_features_json()).unwrap();
    assert_object_keys(
        &features[0],
        &[
            "frame_index",
            "fps",
            "own_hp",
            "opponent_hp",
            "is_match_screen",
            "own_meter_state",
            "opponent_meter_state",
            "left_hp_score",
            "right_hp_score",
            "left_drive_ratio",
            "right_drive_ratio",
            "left_burnout",
            "right_burnout",
            "left_drive_uncertain",
            "right_drive_uncertain",
            "left_hp_raw",
            "right_hp_raw",
            "left_hp_raw_quality",
            "right_hp_raw_quality",
        ],
    );

    let report: Value = serde_json::from_str(&analyzer.finish()).unwrap();
    assert_object_keys(
        &report,
        &[
            "ruleset_version",
            "analyzer_build_id",
            "total_frames",
            "rounds_detected",
            "damage_taken_events",
            "damage_breakdown",
            "weaknesses",
            "practice_items",
            "summary",
            "cards",
            "round_summaries",
            "input_stats",
            "tactic_stats",
            "coverage",
            "analysis_warnings",
        ],
    );

    let timeline: Value = serde_json::from_str(&analyzer.get_timeline()).unwrap();
    assert_object_keys(&timeline, &["left", "right", "video_map"]);
    assert_object_keys(&timeline["left"], &["side", "segments"]);

    let tracked: Value = serde_json::from_str(&analyzer.get_tracked_inputs()).unwrap();
    assert_object_keys(&tracked, &["p1", "p2"]);
    assert_object_keys(
        &tracked["p1"][0],
        &[
            "count",
            "dir",
            "badges",
            "auto",
            "throw",
            "repaired",
            "uncertain",
        ],
    );

    let windows: Value = serde_json::from_str(&analyzer.get_spatial_windows_json()).unwrap();
    assert!(windows.is_array());
}
