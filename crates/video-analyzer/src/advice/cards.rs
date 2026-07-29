use super::*;
use std::cmp::Ordering;

pub(crate) fn build_advice_cards(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    opp: u8,
    own_index: usize,
    own_character: Option<&str>,
    round_summaries: &[RoundSummary],
) -> Vec<AdviceCard> {
    let mut cards = Vec::new();
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
        detect_punish_fail(events, own, own_character),
        detect_punish_missed(events, own, own_character),
        detect_low_conversion(events, own),
        detect_throw_whiff_punished(events, own),
        detect_throw_loop(events, opp),
        detect_early_hits(events, round_summaries, own),
        detect_lead_loss(events, round_summaries, own_index),
    ];
    cards.extend(candidates.into_iter().flatten());
    if let Some(card) = detect_big_hits(events, own, &cards) {
        cards.push(card);
    }
    sort_cards(&mut cards);
    cards
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
