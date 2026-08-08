use super::common::{is_biased, observed_opportunities};
use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};
use crate::match_events::{DefensiveActionKind, EventConfidence, MatchEvents, MinusPressOutcome};

pub(crate) fn detect_press_while_minus(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let selections: Vec<_> = events
        .presses_while_minus
        .iter()
        .filter(|event| {
            event.side == own
                && event.action_kind == DefensiveActionKind::Strike
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
    let escaped = selections.len() - losses.len();
    let selection_percent = selections.len() * 100 / opportunities;
    let common_button = selections
        .iter()
        .map(|event| event.pressed.as_str())
        .max_by_key(|candidate| {
            selections
                .iter()
                .filter(|event| event.pressed.as_str() == *candidate)
                .count()
        })
        .unwrap_or("?");
    Some(AdviceCard {
        id: "press_while_minus".to_string(),
        kind: if biased { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if biased {
            "不利フレーム後の最速打撃に偏っている"
        } else {
            "不利フレーム後の最速打撃で被弾した場面"
        }.to_string(),
        severity: hp_lost + 0.01 * escaped as f32,
        hp_lost: Some(hp_lost),
        description: if biased {
            format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で硬直明け最速打撃を選んでいます。うち {} 回はカウンターで狩られ、合計 {:.0}% の HP を失いました（被弾しなかったのは {} 回）。同じ回答へ偏り、相手の打撃重ねに複数回利用されている点が改善対象です。最も多かった入力は {} でした。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0, escaped, common_button
            )
        } else {
            format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で最速打撃を選び、そのうち {} 回、合計 {:.0}% 被弾しています。この試合で同様の被弾は {} 回です。この結果だけでは、投げを読んだ打撃が偶然負けたのか、回答が偏っているのかは{OBSERVATION_REVIEW_CAVEAT}。最も多かった入力は {} でした。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0, losses.len(), common_button
            )
        },
        practice: if biased {
            "有利を取る打撃と投げをランダム再生し、ガード継続を基準にします。そこへ遅らせ投げ抜け・後退・最速打撃を混ぜ、同じ回答を連続して選ばない練習をしましょう。"
        } else {
            "クリップで、投げを読んで押したのか、連係が終わると思って押したのかを確認します。意図した読みなら単発の失敗として扱い、普段も同じ不利幅で押している場合だけ選択率を下げましょう。"
        }.to_string(),
        evidence: losses.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!("R{} 不利{}Fで{}を押して被弾 -{:.0}%", event.round_no, event.minus_frames, event.pressed, event.drop * 100.0),
        }).collect(),
    })
}
