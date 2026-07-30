use serde_json::Value;
use video_analyzer::{build_match_events_with_context, AnalysisContext, FrameFeatures};

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
fn frame_features_json_keeps_all_browser_fields() {
    let feature = FrameFeatures {
        frame_index: 7,
        fps: 60.0,
        own_hp: 0.8,
        opponent_hp: 0.7,
        is_match_screen: true,
        own_meter_state: Some("active".to_string()),
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.2,
        left_drive_ratio: 0.3,
        right_drive_ratio: 0.4,
        left_burnout: false,
        right_burnout: true,
        left_drive_uncertain: false,
        right_drive_uncertain: true,
        left_super_value: 1.25,
        right_super_value: 2.5,
        left_super_uncertain: false,
        right_super_uncertain: true,
        left_ca_ready: false,
        right_ca_ready: true,
        left_hp_raw: 0.8,
        right_hp_raw: 0.7,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 1.0,
    };

    assert_object_keys(
        &serde_json::to_value(feature).unwrap(),
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
            "left_super_value",
            "right_super_value",
            "left_super_uncertain",
            "right_super_uncertain",
            "left_ca_ready",
            "right_ca_ready",
            "left_hp_raw",
            "right_hp_raw",
            "left_hp_raw_quality",
            "right_hp_raw_quality",
        ],
    );
}

#[test]
fn events_json_keeps_the_serialized_boundary_and_omits_internal_series() {
    let context = AnalysisContext::new("p1");
    let events = build_match_events_with_context(&[], &[], &[], None, &context);
    let value = serde_json::to_value(events).unwrap();

    assert_object_keys(
        &value,
        &[
            "rounds",
            "damage",
            "jumps",
            "throws",
            "throw_actions",
            "drive_impacts",
            "drive_rushes",
            "burnouts",
            "contacts",
            "punishes",
            "reversals",
            "super_arts",
            "guard_breaks",
            "presses_while_minus",
            "minus_situations",
            "projectiles",
            "teleports",
            "compound_threats",
            "segments",
        ],
    );
    for private in ["meter_state", "meter_confidence", "meter_game_frame", "hp"] {
        assert!(value.get(private).is_none());
    }
}
