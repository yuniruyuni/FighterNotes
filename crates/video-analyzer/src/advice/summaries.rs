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
    rounds_detected: u32,
    damage_count: usize,
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
    let summary = match cards
        .iter()
        .find(|card| card.kind == AdviceKind::Diagnosis)
    {
        Some(priority) => format!(
            "{rounds_detected}ラウンド検出、被弾 {damage_count} 件。優先改善: {}",
            priority.title
        ),
        None if cards.is_empty() => format!(
            "{rounds_detected}ラウンド検出、被弾 {damage_count} 件。顕著な改善ポイントは検出されませんでした。"
        ),
        None => format!(
            "{rounds_detected}ラウンド検出、被弾 {damage_count} 件。原因を断定できる改善指摘はなく、要確認: {}",
            cards[0].title
        ),
    };
    (weaknesses, practice_items, summary)
}
