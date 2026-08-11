use crate::match_events::{
    EventConfidence, JumpDirection, JumpOutcome, MatchEvents, JUMP_ATTACK_MAX, JUMP_ATTACK_MIN,
};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};

/// 相手のジャンプに対する自分の対応。
pub fn detect_anti_air(events: &MatchEvents, own: u8, opp: u8) -> Option<AdviceCard> {
    let opp_jumps: Vec<_> = events
        .jumps
        .iter()
        .filter(|jump| {
            jump.side == opp && jump.takeoff_confirmed && jump.direction != JumpDirection::Backward
        })
        .collect();
    let landed: Vec<_> = opp_jumps
        .iter()
        .filter(|jump| jump.outcome == JumpOutcome::LandedHit)
        .collect();
    let anti_aired = opp_jumps
        .iter()
        .filter(|jump| jump.outcome == JumpOutcome::GotHit)
        .count();
    if landed.is_empty() {
        return None;
    }
    let resolved_jumps = landed.len() + anti_aired;
    let repeated =
        landed.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES && landed.len() * 100 >= resolved_jumps * 50;
    let kind = if repeated {
        AdviceKind::Diagnosis
    } else {
        AdviceKind::Observation
    };
    let hp_lost: f32 = landed
        .iter()
        .filter_map(|jump| {
            events.damage.iter().find(|damage| {
                let contact_matches = jump
                    .contact_frame
                    .is_some_and(|contact| damage.start_frame.abs_diff(contact) <= 25);
                damage.victim == own
                    && (contact_matches
                        || (jump.contact_frame.is_none()
                            && damage.start_frame >= jump.frame + JUMP_ATTACK_MIN
                            && damage.start_frame <= jump.frame + JUMP_ATTACK_MAX + 25))
            })
        })
        .map(|damage| damage.drop)
        .sum::<f32>()
        .max(0.0);
    let neutral = opp_jumps.len() - landed.len() - anti_aired;
    Some(AdviceCard {
        id: "anti_air".to_string(),
        kind,
        confidence: EventConfidence::High,
        title: match kind {
            AdviceKind::Diagnosis => "飛び込みを繰り返し通している",
            _ => "飛び込みを通した場面",
        }.to_string(),
        severity: hp_lost + 0.02 * landed.len() as f32,
        hp_lost: Some(hp_lost),
        description: match kind {
            AdviceKind::Diagnosis => format!(
                "相手の前・垂直ジャンプ {} 回中、空中で迎撃できたのは {} 回、飛び込みを通されたのは {} 回です（残り {} 回はどちらでもないジャンプ）。通された割合が高く、同じ被弾が複数回あるため対空を改善候補とします。失った HP は合計 {:.0}% です。",
                opp_jumps.len(), anti_aired, landed.len(), neutral, hp_lost * 100.0
            ),
            _ => format!(
                "相手の前・垂直ジャンプ {} 回中、空中で迎撃できたのは {} 回、飛び込みを通されたのは {} 回です（残り {} 回）。この試合で同様に飛びを通したのは {} 回、失った HP は合計 {:.0}% です。この件数だけでは、地上へ意識を割いた読み合いの結果か、対空が遅れる傾向かは{OBSERVATION_REVIEW_CAVEAT}。",
                opp_jumps.len(), anti_aired, landed.len(), neutral, landed.len(), hp_lost * 100.0
            ),
        },
        practice: match kind {
            AdviceKind::Diagnosis => "トレーニングモードで前ジャンプを2〜3種類記録してランダム再生し、対空を20回連続で成功させます。まずは地上への意識を少し下げてでも、見てから迎撃する練習を優先しましょう。",
            _ => "クリップで、飛びが見えていなかったのか、別の行動中で対空できなかったのかを確認します。普段も同じ距離の飛びを通している場合だけ、対空練習の優先度を上げましょう。",
        }.to_string(),
        evidence: landed.iter().map(|jump| EvidenceClip {
            frame: jump.frame,
            end_frame: None,
            label: format!("R{} 飛び込みを通した", jump.round_no),
        }).collect(),
    })
}
