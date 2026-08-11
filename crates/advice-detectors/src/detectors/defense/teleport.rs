use crate::match_events::{
    DefenseResponseKind, DpReachability, EventConfidence, MatchEvents, TeleportContext,
    ThreatOutcome,
};
use crate::{AdviceCard, AdviceKind, EvidenceClip};

/// A naked attacking teleport that a calibrated anti-air can reach.
/// Unknown spatial reach deliberately abstains.
pub fn detect_teleport_defense(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let missed: Vec<_> = events
        .teleports
        .iter()
        .filter(|teleport| {
            teleport.defender == own
                && teleport.context == TeleportContext::NakedAttack
                && teleport.defender_actionable
                && teleport.followup_attack_frame.is_some()
                && teleport.damage > 0.0
                && teleport.outcome == ThreatOutcome::Hit
                && teleport.dp_reachability == DpReachability::Confirmed
                && !teleport.response.as_ref().is_some_and(|response| {
                    matches!(
                        response.kind,
                        DefenseResponseKind::Parry | DefenseResponseKind::Invincible
                    )
                })
        })
        .collect();
    if missed.is_empty() {
        return None;
    }
    let hp_lost: f32 = missed.iter().map(|teleport| teleport.damage).sum();
    Some(AdviceCard {
        id: "teleport_defense".to_string(),
        kind: AdviceKind::Diagnosis,
        confidence: EventConfidence::High,
        title: match missed.len() {
            1 => "裸テレポートを迎撃できなかった場面",
            _ => "裸テレポートへの迎撃が遅れている",
        }
        .to_string(),
        severity: hp_lost + 0.02 * missed.len() as f32,
        hp_lost: Some(hp_lost),
        description: format!(
            "飛び道具を重ねていないテレポート攻撃に対し、行動可能かつ昇竜系の対空が届くことを確認できたのに迎撃できなかった場面が {} 回、合計 {:.0}% あります。弾と挟まれる複合連係や、硬直中のテレポートはこの件数に含めていません。",
            missed.len(),
            hp_lost * 100.0
        ),
        practice: "裸テレポートと飛び道具を重ねたテレポートを別スロットに記録してランダム再生します。裸テレポートだけを昇竜系対空で迎撃し、飛び道具が残る連係ではパリィ・ガードへ切り替える練習をします。".to_string(),
        evidence: missed
            .iter()
            .map(|teleport| EvidenceClip {
                frame: teleport.input_frame,
                end_frame: teleport.followup_attack_frame.map(|frame| frame + 30),
                label: format!("R{} 裸テレポートを迎撃できた場面", teleport.round_no),
            })
            .collect(),
    })
}
