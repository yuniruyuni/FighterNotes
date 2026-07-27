use super::support::*;

#[test]
fn legacy_input_segment_defaults_to_direct_observation() {
    let segment: InputSegment = serde_json::from_value(serde_json::json!({
        "start_frame": 10,
        "end_frame": 12,
        "dir": "N",
        "badges": ["弱"],
        "auto": false,
        "throw": false
    }))
    .expect("旧 InputSegment を読める");

    assert!(segment.evidence.has_direct_observation());
    assert_eq!(segment.evidence.repaired_frames, 0);
}
