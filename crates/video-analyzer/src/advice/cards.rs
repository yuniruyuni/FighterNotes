use super::*;
use std::cmp::Ordering;

pub(crate) fn build_advice_cards(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    own_index: usize,
    own_character: Option<&str>,
    round_summaries: &[RoundSummary],
    coverage: &AnalysisCoverage,
) -> Vec<AdviceCard> {
    let mut cards = Vec::new();
    let opp = 3 - own;
    let candidates = [
        detect_layered_defense(events, own),
        detect_teleport_defense(events, own),
        detect_anti_air(events, own, opp),
        detect_own_jumps(events, own),
        detect_burnout(events, own),
        detect_committed_button_vs_di(events, own, own_index),
        detect_mashing(features, events, own, own_index),
        detect_press_while_minus(events, own),
        detect_throw_while_minus(events, own),
        detect_guard_break(events, own),
        detect_reversal_punished(events, own),
        detect_low_scaling_super(events, own),
        detect_punish_fail(events, own, own_character),
        detect_punish_missed(events, own, own_character),
        detect_low_conversion(events, own),
        detect_throw_interrupted_by_invincible(events, own),
        detect_throw_whiff_punished(events, own),
        detect_throw_loop(events, opp),
        detect_early_hits(events, round_summaries, own),
        detect_lead_loss(events, round_summaries, own_index),
    ];
    cards.extend(candidates.into_iter().flatten());
    cards.retain(|card| card_has_required_coverage(card, coverage));
    if let Some(card) = detect_big_hits(events, own, &cards) {
        if card_has_required_coverage(&card, coverage) {
            cards.push(card);
        }
    }
    sort_cards(&mut cards);
    cards
}

fn card_has_required_coverage(card: &AdviceCard, coverage: &AnalysisCoverage) -> bool {
    let total = coverage.detector_match_frames;
    let own_input = detector_coverage_is_sufficient(coverage.own_input_observed_frames, total);
    let opponent_input =
        detector_coverage_is_sufficient(coverage.opponent_input_observed_frames, total);
    let own_hp = detector_coverage_is_sufficient(coverage.own_hp_reliable_frames, total);
    let opponent_hp = detector_coverage_is_sufficient(coverage.opponent_hp_reliable_frames, total);
    let both_meter = detector_coverage_is_sufficient(
        coverage
            .own_meter_mapped_frames
            .min(coverage.opponent_meter_mapped_frames),
        total,
    );
    let own_drive = detector_coverage_is_sufficient(coverage.own_drive_reliable_frames, total);
    let own_super = super_coverage_is_sufficient(coverage.own_super_reliable_frames, total);
    let spatial = detector_coverage_is_sufficient(
        coverage.spatial_sampled_frames,
        coverage.spatial_candidate_frames,
    ) && spatial_coverage_is_sufficient(
        coverage.spatial_usable_frames,
        coverage.spatial_candidate_frames,
    );
    let attack_info = detector_coverage_is_sufficient(
        coverage.attack_damage_linked,
        coverage.attack_damage_events,
    );

    match card.id.as_str() {
        // 入力履歴から機会と実行を確定するカード。
        "anti_air" => opponent_input && own_hp && opponent_hp,
        "own_jumps" => own_input && own_hp && opponent_hp,
        "throw_loop" => opponent_input && own_hp,
        // 入力とフレームメーターの時系列が揃って初めて因果を述べられるカード。
        "committed_button_vs_di" => own_input && opponent_input && both_meter && own_hp,
        "mashing"
        | "press_while_minus"
        | "throw_while_minus"
        | "guard_break"
        | "throw_interrupted_by_invincible"
        | "throw_whiff_punished" => own_input && both_meter && own_hp,
        // 発生・硬直・接触をフレームメーターから作るカード。
        "reversal_punished" => both_meter && own_hp,
        "low_conversion" => both_meter && opponent_hp,
        // 到達距離まで断定するカードは候補区間の空間観測も必要。
        "punish_fail" => both_meter && spatial && own_hp,
        "teleport_defense" => both_meter && spatial && own_hp,
        "punish_missed" => both_meter && spatial && own_hp,
        // 複合攻撃の成立自体はmeter/contactから確定し、距離は使わない。
        "layered_defense" => both_meter && own_hp,
        "burnout" => own_drive && own_hp && opponent_hp,
        "low_scaling_super" => own_super && attack_info && opponent_hp,
        "early_hits" | "big_hits" => own_hp,
        "lead_loss" => own_hp && opponent_hp,
        _ => true,
    }
}

fn sort_cards(cards: &mut [AdviceCard]) {
    let kind_rank = |kind| match kind {
        AdviceKind::Diagnosis => 2,
        AdviceKind::Observation => 1,
        AdviceKind::Statistic => 0,
    };
    let confidence_rank = |confidence| match confidence {
        EventConfidence::High => 2,
        EventConfidence::Medium => 1,
        EventConfidence::Low => 0,
    };
    cards.sort_by(|left, right| {
        kind_rank(right.kind)
            .cmp(&kind_rank(left.kind))
            .then_with(|| confidence_rank(right.confidence).cmp(&confidence_rank(left.confidence)))
            .then_with(|| {
                right
                    .severity
                    .partial_cmp(&left.severity)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.id.cmp(&right.id))
    });
}
