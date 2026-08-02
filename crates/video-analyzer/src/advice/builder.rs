use super::*;

fn analyzer_build_id() -> String {
    let revision = option_env!("FIGHTER_NOTES_BUILD_SHA").unwrap_or("dev");
    format!("{}+{}", env!("CARGO_PKG_VERSION"), revision)
}

/// 旧API互換。相手キャラクターが不明なため、打撃属性の技照合は行わない。
pub fn build_report(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own_side: &str,
    own_char: Option<&str>,
) -> AdviceReport {
    let context = crate::context::AnalysisContext::from_characters(own_side, own_char, None);
    build_report_with_context(features, events, &context)
}

/// イベント層と対戦メタデータからアドバイスレポートを生成する。
pub fn build_report_with_context(
    features: &[FrameFeatures],
    events: &MatchEvents,
    context: &crate::context::AnalysisContext,
) -> AdviceReport {
    let own = if context.own_side() == "p2" { 2 } else { 1 };
    let opp = 3 - own;
    let own_index = own as usize - 1;
    let total_frames = features
        .iter()
        .map(|feature| feature.frame_index)
        .max()
        .unwrap_or(0)
        + 1;
    let rounds_detected = events.rounds.len() as u32;

    let damage_taken_events = build_damage_taken_events(events, own);
    let round_summaries = build_round_summaries(events, own, opp);
    let (coverage, analysis_warnings) =
        build_coverage(features, events, own_index, &round_summaries);
    let input_stats = detector_coverage_is_sufficient(
        coverage.own_input_observed_frames,
        coverage.detector_match_frames,
    )
    .then(|| build_input_stats(features, events, own, own_index))
    .flatten();
    let tactic_stats = build_tactic_stats(features, events, own, opp);
    let (cards, suppressed_cards) = build_advice_cards(
        features,
        events,
        own,
        own_index,
        context.own_character(),
        &round_summaries,
        &coverage,
    );

    let mut damage_breakdown =
        damage_origins::build_damage_breakdown(features, events, own, context.opponent_character());
    damage_origins::apply_advice_contexts(&mut damage_breakdown, &cards);
    let (weaknesses, practice_items, summary) = build_compatibility_summary(
        &cards,
        &suppressed_cards,
        rounds_detected,
        damage_taken_events.len(),
        &coverage,
    );

    AdviceReport {
        ruleset_version: RULESET_VERSION,
        analyzer_build_id: analyzer_build_id(),
        total_frames,
        rounds_detected,
        damage_taken_events,
        damage_breakdown,
        weaknesses,
        practice_items,
        summary,
        cards,
        suppressed_cards,
        round_summaries,
        input_stats,
        tactic_stats,
        coverage,
        analysis_warnings,
    }
}
