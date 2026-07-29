use super::Analyzer;

#[test]
fn set_characters_keeps_legacy_own_opponent_mapping() {
    let mut analyzer = Analyzer::new("p2");
    analyzer.set_characters("BLANKA", "DHALSIM");

    assert_eq!(
        analyzer.analysis_context.p1.character.as_deref(),
        Some("DHALSIM")
    );
    assert_eq!(
        analyzer.analysis_context.p2.character.as_deref(),
        Some("BLANKA")
    );
    assert_eq!(analyzer.analysis_context.own_character(), Some("BLANKA"));
}

#[test]
fn analysis_context_json_preserves_player_metadata() {
    let mut analyzer = Analyzer::new("p1");
    analyzer
        .set_analysis_context(
            r#"{
                "ownSide":"p2",
                "p1":{"character":"KEN","controlType":"classic"},
                "p2":{"character":"DHALSIM","controlType":"modern"},
                "battleVersion":"2026.06"
            }"#,
        )
        .unwrap();

    assert_eq!(analyzer.analysis_context.own_side(), "p1");
    assert_eq!(
        analyzer.analysis_context.p2.control_type.as_deref(),
        Some("modern")
    );
    assert_eq!(
        analyzer.analysis_context.battle_version.as_deref(),
        Some("2026.06")
    );
}

#[test]
fn split_meter_session_matches_combined_analysis() {
    let mut combined = Analyzer::new("p1");
    let mut meter = Analyzer::new("p1");
    let mut result = Analyzer::new("p1");

    for frame_index in 0..3 {
        combined.analyze_meter_inplace(1920, 1080, frame_index);
        combined.push_hud_features_inplace(1920, 1080, frame_index);
        combined.analyze_input_inplace(1920, 1080, frame_index);

        meter.analyze_meter_inplace(1920, 1080, frame_index);
        result.push_hud_features_inplace(1920, 1080, frame_index);
        result.analyze_input_inplace(1920, 1080, frame_index);
    }

    let meter_timeline = meter.finish_meter_timeline();
    result.set_meter_timeline(&meter_timeline).unwrap();

    assert_eq!(result.finish(), combined.finish());
    assert_eq!(result.get_timeline(), combined.get_timeline());
    assert_eq!(result.get_features_json(), combined.get_features_json());
    assert_eq!(result.get_tracked_inputs(), combined.get_tracked_inputs());
    assert_eq!(
        result.get_spatial_windows_json(),
        combined.get_spatial_windows_json()
    );
}
