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
