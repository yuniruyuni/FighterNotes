use super::*;
use crate::match_events::AttackDamageConsistency;

pub(crate) fn build_coverage(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own_index: usize,
    round_summaries: &[RoundSummary],
) -> (AnalysisCoverage, Vec<String>) {
    let match_frames = features
        .iter()
        .filter(|feature| feature.is_match_screen)
        .count() as u32;
    let analyzed_match_frames = features
        .iter()
        .filter(|feature| {
            feature.is_match_screen
                && crate::match_events::round_of(&events.rounds, feature.frame_index).is_some()
        })
        .count() as u32;
    let input_segments = events.segments[own_index].len() as u32;
    let analyzed_input_segments = events.segments[own_index]
        .iter()
        .filter(|segment| {
            events.rounds.iter().any(|round| {
                segment.start_frame <= round.end_frame && segment.end_frame >= round.start_frame
            })
        })
        .count() as u32;
    let own = own_index as u8 + 1;
    let attack_damage_events = events
        .damage
        .iter()
        .filter(|damage| damage.victim == own)
        .count() as u32;
    let own_attack_evidence: Vec<_> = events
        .attack_evidence
        .damage
        .iter()
        .filter(|evidence| evidence.victim == own)
        .collect();
    let coverage = AnalysisCoverage {
        match_frames,
        analyzed_match_frames,
        input_segments,
        analyzed_input_segments,
        attack_damage_events,
        attack_damage_linked: own_attack_evidence.len() as u32,
        attack_damage_consistent: own_attack_evidence
            .iter()
            .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Consistent)
            .count() as u32,
        attack_damage_mismatched: own_attack_evidence
            .iter()
            .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Mismatch)
            .count() as u32,
    };
    let warnings = analysis_warnings(&coverage, events.rounds.len(), round_summaries);
    (coverage, warnings)
}

fn analysis_warnings(
    coverage: &AnalysisCoverage,
    rounds_detected: usize,
    round_summaries: &[RoundSummary],
) -> Vec<String> {
    let mut warnings = Vec::new();
    if rounds_detected == 0 {
        warnings.push("ラウンド境界を確定できなかったため、戦術統計を表示できません。".to_string());
    } else if coverage.match_frames > 0
        && u64::from(coverage.analyzed_match_frames) * 100 < u64::from(coverage.match_frames) * 70
    {
        let ratio =
            u64::from(coverage.analyzed_match_frames) * 100 / u64::from(coverage.match_frames);
        warnings.push(format!(
            "試合画面のうち解析ラウンドへ割り当てられた範囲は {ratio}% です。未検出ラウンドがないか確認してください。"
        ));
    }
    if round_summaries.iter().any(|round| round.won.is_none()) {
        warnings.push(
            "勝敗を確定できないラウンドがあります。ラウンド表の対象シーンを確認してください。"
                .to_string(),
        );
    }
    if coverage.attack_damage_mismatched > 0 {
        warnings.push(format!(
            "ゲーム内ダメージ表示とHPバー由来の減少量が一致しない被弾が {} 件あります。HP値は自動補正せず、ゲーム内表示を補助証拠として記録します。",
            coverage.attack_damage_mismatched
        ));
    }
    warnings
}
