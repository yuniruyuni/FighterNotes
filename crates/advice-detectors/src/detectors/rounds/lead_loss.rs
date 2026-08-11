use crate::match_events::{EventConfidence, MatchEvents};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, RoundSummary, LEAD_MARGIN, OBSERVATION_REVIEW_CAVEAT,
};

pub fn detect_lead_loss(
    events: &MatchEvents,
    rounds: &[RoundSummary],
    own_index: usize,
) -> Option<AdviceCard> {
    let mut losses = Vec::new();
    for summary in rounds {
        if summary.won == Some(false) {
            if let Some(round) = events
                .rounds
                .iter()
                .find(|round| round.round_no == summary.round_no)
            {
                let own_hp = &events.hp[own_index];
                let opponent_hp = &events.hp[1 - own_index];
                let len = own_hp.len().min(opponent_hp.len());
                let start = round.start_frame as usize;
                if let std::cmp::Ordering::Less = start.cmp(&len) {
                    let end = (round.end_frame as usize).min(len.saturating_sub(1));
                    let max_lead = (start..=end)
                        .map(|frame| own_hp[frame] - opponent_hp[frame])
                        .fold(f32::MIN, f32::max);
                    if max_lead >= LEAD_MARGIN {
                        let peak = (start..=end)
                            .rev()
                            .find(|&frame| own_hp[frame] - opponent_hp[frame] >= max_lead - 0.001)
                            .unwrap_or(start);
                        let flipped = own_hp
                            .iter()
                            .zip(opponent_hp)
                            .enumerate()
                            .find_map(|(frame, (&own, &opponent))| {
                                (frame >= peak && frame < end && opponent > own).then_some(frame)
                            })
                            .unwrap_or(end);
                        losses.push((summary, peak as u32, flipped as u32));
                    }
                }
            }
        }
    }
    if losses.is_empty() {
        return None;
    }
    Some(AdviceCard {
        id: "lead_loss".to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::Medium,
        title: "大きなリードから逆転された場面".to_string(),
        severity: 0.15 * losses.len() as f32,
        hp_lost: Some(
            losses
                .iter()
                .map(|(_, peak, flipped)| {
                    let hp = &events.hp[own_index];
                    let peak_hp = hp.get(*peak as usize).copied().unwrap_or(0.0);
                    let flipped_hp = hp.get(*flipped as usize).copied().unwrap_or(0.0);
                    (peak_hp - flipped_hp).max(0.0)
                })
                .sum(),
        ),
        description: format!(
            "HP リード 30% 以上を持ちながら落としたラウンドが {} 回あります。逆転された事実だけでは、攻め継続・後退・ガードなど特定の選択が悪かったとは{OBSERVATION_REVIEW_CAVEAT}。最大リード以降に同じ行動で複数回被弾していないかを確認するための場面一覧です。",
            losses.len()
        ),
        practice: "最大リード以降の各被弾について、自分の直前行動を記録します。同じ接近・同じ暴れ・同じゲージ使用が繰り返されていた場合だけ、次の対戦でその選択率を下げましょう。".to_string(),
        evidence: losses.iter().map(|(round, peak, flipped)| EvidenceClip {
            frame: *peak,
            end_frame: Some(*flipped),
            label: format!("R{} 最大リード→逆転", round.round_no),
        }).collect(),
    })
}
