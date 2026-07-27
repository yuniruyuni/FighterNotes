use super::common::{is_biased, observed_opportunities};
use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};
use crate::match_events::{DefensiveActionKind, EventConfidence, MatchEvents, MinusPressOutcome};

pub(crate) fn detect_throw_while_minus(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let selections: Vec<_> = events
        .presses_while_minus
        .iter()
        .filter(|event| {
            event.side == own
                && event.action_kind == DefensiveActionKind::Throw
                && event.confidence == EventConfidence::High
        })
        .collect();
    let losses: Vec<_> = selections
        .iter()
        .copied()
        .filter(|event| event.outcome == MinusPressOutcome::CounterHit)
        .collect();
    if losses.is_empty() {
        return None;
    }
    let opportunities = observed_opportunities(events, own, selections.len());
    let biased = is_biased(opportunities, selections.len(), losses.len());
    let hp_lost: f32 = losses.iter().map(|event| event.drop).sum();
    let selection_percent = selections.len() * 100 / opportunities;
    Some(AdviceCard {
        id: "throw_while_minus".to_string(),
        kind: if biased { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if biased {
            "不利フレーム後の最速投げに偏っている"
        } else {
            "不利フレーム後の最速投げで被弾した場面"
        }.to_string(),
        severity: hp_lost + 0.01 * (selections.len() - losses.len()) as f32,
        description: if biased {
            format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で最速投げを選んでいます。うち {} 回は打撃に負け、合計 {:.0}% の HP を失いました。同じ回答への偏りが複数回利用されたことを指摘しています。投げ抜けではなく、自分から最速で投げた場面だけを数えています。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0
            )
        } else {
            format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で最速投げを選び、そのうち {} 回、合計 {:.0}% 被弾しています。この試合で同様の被弾は {} 回です。この件数だけでは、相手の投げを読んだ回答が打撃に負けたのか、最速投げへ偏っているのかは{OBSERVATION_REVIEW_CAVEAT}。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0, losses.len()
            )
        },
        practice: if biased {
            "相手の有利連係を記録し、ガード継続・遅らせ投げ抜け・後退・最速投げを順に試します。同じ回答を連続して選ばず、相手の打撃と投げの比率に合わせて散らしましょう。"
        } else {
            "クリップで、投げを読んで自分から投げたのかを確認します。意図した読みなら単発の失敗として扱い、同じ不利状況で毎回投げている場合だけ回答を散らしましょう。"
        }.to_string(),
        evidence: losses.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!("R{} 不利{}Fから最速投げで被弾 -{:.0}%", event.round_no, event.minus_frames, event.drop * 100.0),
        }).collect(),
    })
}
