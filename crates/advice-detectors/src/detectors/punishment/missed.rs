use super::options::missed_option_text;
use crate::match_events::{EventConfidence, MatchEvents, PunishOutcome, PunishReachability};
use crate::{AdviceCard, AdviceKind, EvidenceClip};

pub fn detect_punish_missed(
    events: &MatchEvents,
    own: u8,
    own_character: Option<&str>,
) -> Option<AdviceCard> {
    let missed: Vec<_> = events
        .punishes
        .iter()
        .filter(|punish| {
            punish.side == own
                && punish.outcome == PunishOutcome::Missed
                && punish.reachability == PunishReachability::Confirmed
        })
        .collect();
    if missed.is_empty() {
        return None;
    }
    let min_advantage = missed
        .iter()
        .map(|punish| punish.advantage)
        .min()
        .unwrap_or(0);
    let option_text = missed_option_text(own_character, min_advantage);
    Some(AdviceCard {
        id: "punish_missed".to_string(),
        kind: AdviceKind::Diagnosis,
        confidence: EventConfidence::High,
        title: if missed.len() == 1 {
            "確定反撃を見逃した場面"
        } else {
            "確定反撃を繰り返し見逃している"
        }.to_string(),
        severity: 0.04 * missed.len() as f32,
        // 損失は機会費用であり、この指摘が原因で失った HP ではない。
        hp_lost: None,
        description: format!(
            "相手の技をガードした後、フレーム上の反撃猶予があり、位置解析でも近距離だったのに反撃していない場面が {} 回あります。相手の危険な技を覚えて、ガードしたら反撃する意識を持ちましょう。{}",
            missed.len(), option_text
        ),
        practice: "対戦相手がよく振る技のうち、ガードして確反が取れるものを 2-3 個に絞って覚えましょう。トレモでその技をガード → 最速で確反、を反復して実戦でも無意識に確反をとれるようにしましょう。".to_string(),
        evidence: missed.iter().map(|punish| EvidenceClip {
            frame: punish.frame,
            end_frame: None,
            label: format!("R{} 確反見逃し +{}F / 近距離確認", punish.round_no, punish.advantage),
        }).collect(),
    })
}
