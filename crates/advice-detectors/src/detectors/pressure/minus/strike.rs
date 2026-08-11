use super::common::observed_opportunities;
use crate::decisions::{
    collect_decisions, losses as decision_losses, selections as decision_selections,
    DecisionOption, DecisionSituation,
};
use crate::detectors::pressure::common::is_biased;
use crate::match_events::{EventConfidence, MatchEvents};
use crate::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};

pub fn detect_press_while_minus(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let decisions = collect_decisions(events, own);
    let selections = decision_selections(
        &decisions,
        DecisionSituation::Disadvantage,
        DecisionOption::Strike,
    );
    let losses = decision_losses(&selections);
    if losses.is_empty() {
        return None;
    }
    let opportunities = observed_opportunities(events, own, selections.len());
    let biased = is_biased(opportunities, selections.len(), losses.len());
    let kind = if biased {
        AdviceKind::Diagnosis
    } else {
        AdviceKind::Observation
    };
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
        kind,
        confidence: EventConfidence::High,
        title: match kind {
            AdviceKind::Diagnosis => "不利フレーム後の最速打撃に偏っている",
            _ => "不利フレーム後の最速打撃で被弾した場面",
        }.to_string(),
        severity: hp_lost + 0.01 * escaped as f32,
        hp_lost: Some(hp_lost),
        description: match kind {
            AdviceKind::Diagnosis => format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で硬直明け最速打撃を選んでいます。うち {} 回はカウンターで狩られ、合計 {:.0}% の HP を失いました（被弾しなかったのは {} 回）。同じ回答へ偏り、相手の打撃重ねに複数回利用されている点が改善対象です。最も多かった入力は {} でした。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0, escaped, common_button
            ),
            _ => format!(
                "入力まで確認できた不利フレーム後の判断 {} 回中、{} 回（{}%）で最速打撃を選び、そのうち {} 回、合計 {:.0}% 被弾しています。この試合で同様の被弾は {} 回です。この結果だけでは、投げを読んだ打撃が偶然負けたのか、回答が偏っているのかは{OBSERVATION_REVIEW_CAVEAT}。最も多かった入力は {} でした。",
                opportunities, selections.len(), selection_percent, losses.len(), hp_lost * 100.0, losses.len(), common_button
            ),
        },
        practice: match kind {
            AdviceKind::Diagnosis => "有利を取る打撃と投げをランダム再生し、ガード継続を基準にします。そこへ遅らせ投げ抜け・後退・最速打撃を混ぜ、同じ回答を連続して選ばない練習をしましょう。",
            _ => "クリップで、投げを読んで押したのか、連係が終わると思って押したのかを確認します。意図した読みなら単発の失敗として扱い、普段も同じ不利幅で押している場合だけ選択率を下げましょう。",
        }.to_string(),
        evidence: losses.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!("R{} 不利{}Fで{}を押して被弾 -{:.0}%", event.round_no, event.frames, event.pressed, event.drop * 100.0),
        }).collect(),
    })
}
