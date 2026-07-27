use super::model::MashHit;
use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::EventConfidence;

pub(super) fn build(hits: Vec<MashHit>) -> Option<AdviceCard> {
    if hits.is_empty() {
        return None;
    }
    let repeated = hits.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = hits.iter().map(|hit| hit.drop).sum();
    let common_input = hits
        .iter()
        .map(|hit| hit.input.as_str())
        .max_by_key(|candidate| hits.iter().filter(|hit| hit.input == *candidate).count())
        .unwrap_or("ボタン");
    Some(AdviceCard {
        id: "mashing".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: if hits.iter().all(|hit| hit.meter_confirmed) {
            EventConfidence::High
        } else {
            EventConfidence::Medium
        },
        title: if repeated {
            "守勢でボタンを押して繰り返し被弾している"
        } else {
            "守勢でボタンを押して被弾した場面"
        }.to_string(),
        severity: hp_lost,
        description: if repeated {
            format!(
                "相手の攻めを受けている途中でボタンを押し、大きく被弾した場面が {} 回、合計 {:.0}% あります。同じ防御回答で複数回損失が出ているため改善候補です。最も多かった入力は {} でした。打撃を押すこと自体ではなく、相手の投げとの読み合いを含めて同じ回答が続いていないかを見直してください。",
                hits.len(), hp_lost * 100.0, common_input
            )
        } else {
            format!(
                "相手の攻めを受けている途中で {} を押し、{:.0}% 被弾した場面が1回あります。打撃は投げへの回答にもなるため、この1回だけでは悪い暴れか、読み合いが噛み合わなかっただけかは{OBSERVATION_REVIEW_CAVEAT}。この試合で同様の被弾は1回です。",
                common_input, hp_lost * 100.0
            )
        },
        practice: if repeated {
            "相手の固めを記録し、まずボタンを押さずガードだけで受け切ります。慣れたら連係の切れ目・投げ・様子見をランダム再生し、最速打撃を同じタイミングで連続して選ばない練習へ進みます。"
        } else {
            "クリップで、投げを読んで押したのか、連係の切れ目だと思って押したのかを確認します。判断に理由があれば単発の失敗として扱い、普段も同じタイミングで押している場合だけ回答を散らしましょう。"
        }.to_string(),
        evidence: hits.iter().map(|hit| EvidenceClip {
            frame: hit.press_frame,
            end_frame: Some(hit.damage_end_frame),
            label: format!("R{} {}入力→大被弾 -{:.0}%", hit.round_no, hit.input, hit.drop * 100.0),
        }).collect(),
    })
}
