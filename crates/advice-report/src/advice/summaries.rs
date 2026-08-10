use super::*;

pub(crate) fn build_damage_taken_events(events: &MatchEvents, own: u8) -> Vec<DamageTakenEvent> {
    events
        .damage
        .iter()
        .filter(|damage| damage.victim == own)
        .map(|damage| DamageTakenEvent {
            frame: damage.start_frame,
            own_hp_before: damage.hp_before,
            own_hp_after: damage.hp_after,
            hp_drop: damage.drop,
            meter_state: None,
        })
        .collect()
}

pub(crate) fn build_round_summaries(
    events: &MatchEvents,
    own: u8,
    opponent: u8,
) -> Vec<RoundSummary> {
    events
        .rounds
        .iter()
        .map(|round| {
            let own_damage: Vec<_> = events
                .damage
                .iter()
                .filter(|damage| damage.round_no == round.round_no && damage.victim == own)
                .collect();
            let opponent_lost = events
                .damage
                .iter()
                .filter(|damage| damage.round_no == round.round_no && damage.victim == opponent)
                .map(|damage| damage.drop)
                .sum();
            let (own_hp_end, opp_hp_end) = if own == 1 {
                (round.p1_hp_end, round.p2_hp_end)
            } else {
                (round.p2_hp_end, round.p1_hp_end)
            };
            RoundSummary {
                round_no: round.round_no,
                start_frame: round.start_frame,
                end_frame: round.end_frame,
                won: round.winner.map(|winner| winner == own),
                own_hp_end,
                opp_hp_end,
                own_hp_lost: own_damage.iter().map(|damage| damage.drop).sum(),
                opp_hp_lost: opponent_lost,
                own_hits_taken: own_damage.len() as u32,
                early_hit: own_damage
                    .iter()
                    .any(|damage| damage.start_frame < round.start_frame + EARLY_HIT_FRAMES),
                own_burnouts: events
                    .burnouts
                    .iter()
                    .filter(|burnout| burnout.round_no == round.round_no && burnout.side == own)
                    .count() as u32,
                detection_confidence: if round.winner.is_some() {
                    "high".to_string()
                } else {
                    "medium".to_string()
                },
            }
        })
        .collect()
}

pub(crate) fn build_compatibility_summary(
    cards: &[AdviceCard],
    suppressed_cards: &[SuppressedAdviceCard],
    rounds_detected: u32,
    damage_count: usize,
    coverage: &AnalysisCoverage,
) -> (Vec<Weakness>, Vec<String>, String) {
    let weaknesses = cards
        .iter()
        .map(|card| Weakness {
            category: card.id.clone(),
            description: card.description.clone(),
            frequency: card.evidence.len() as u32,
        })
        .collect();
    let practice_items = cards.iter().map(|card| card.practice.clone()).collect();
    let own_hp_available = coverage
        .availability
        .as_ref()
        .is_none_or(|availability| availability.own_hp.is_available());
    let scope = if own_hp_available {
        format!("{rounds_detected}ラウンド検出、被弾 {damage_count} 件。")
    } else {
        format!("{rounds_detected}ラウンド検出。HPバーの認識率不足により、被弾件数は確認不能です。")
    };
    let supports_no_findings_claim = coverage
        .availability
        .as_ref()
        .is_none_or(AnalysisAvailability::supports_no_findings_claim);
    let summary = match cards
        .iter()
        .find(|card| card.kind == AdviceKind::Diagnosis)
    {
        Some(priority) => format!("{scope} 優先改善: {}", priority.title),
        None if cards.is_empty() && !suppressed_cards.is_empty() => format!(
            "{scope} {}件の指摘候補は証拠不足で確認不能です。改善点なしとは判定していません。",
            suppressed_cards.len()
        ),
        None if cards.is_empty() && !supports_no_findings_claim => format!(
            "{scope} 認識率不足のため、改善ポイントを十分に判定できませんでした。改善点なしとは判定していません。"
        ),
        None if cards.is_empty() => format!(
            "{scope} 顕著な改善ポイントは検出されませんでした。"
        ),
        None => format!(
            "{scope} 原因を断定できる改善指摘はなく、要確認: {}",
            cards[0].title
        ),
    };
    let summary = if !cards.is_empty() && !suppressed_cards.is_empty() {
        format!(
            "{summary} なお、{}件の候補は証拠不足で確認不能です。",
            suppressed_cards.len()
        )
    } else {
        summary
    };
    (weaknesses, practice_items, summary)
}

#[cfg(test)]
mod tests;
