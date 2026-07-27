use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{EventConfidence, MatchEvents};

pub(crate) fn detect_reversal_punished(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let reversals: Vec<_> = events
        .reversals
        .iter()
        .filter(|event| event.side == own && event.confidence == EventConfidence::High)
        .collect();
    if reversals.is_empty() {
        return None;
    }
    let repeated = reversals.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = reversals.iter().map(|event| event.drop).sum();
    Some(AdviceCard {
        id: "reversal_punished".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if repeated {
            "無敵技という防御回答を繰り返し狩られている"
        } else {
            "無敵技を狩られた場面"
        }.to_string(),
        severity: hp_lost + 0.02 * reversals.len() as f32,
        description: if repeated {
            format!(
                "無敵技（昇竜・SAなど）が通らず、後隙を狩られた場面が {} 回、合計 {:.0}% あります。技名までは確定していませんが、無敵技という同じ防御回答で複数回損失が出ているため、選択頻度を見直す候補です。",
                reversals.len(), hp_lost * 100.0
            )
        } else {
            format!(
                "無敵技（昇竜・SAなど）が通らず、後隙を狩られて {:.0}% 被弾した場面が1回あります。無敵技は打撃重ねへの正しい回答にもなるため、この1回だけでは読み負けか選択の偏りかを{OBSERVATION_REVIEW_CAVEAT}。この試合で同様の被弾は1回です。",
                hp_lost * 100.0
            )
        },
        practice: if repeated {
            "起き攻めや固めを記録し、ガード・遅らせ投げ抜け・バックステップ・無敵技を混ぜます。無敵技を同じ局面で連続して選ばず、相手が打撃重ねに偏ったと確認したときだけ比率を上げましょう。"
        } else {
            "クリップで、相手の打撃重ねを読んで撃ったのか、苦しくなって無意識に撃ったのかを確認します。意図した読みなら単発の失敗として扱い、普段も同じ局面で撃っている場合だけ頻度を下げましょう。"
        }.to_string(),
        evidence: reversals.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!(
                "R{} 無敵技を{}られ -{:.0}%",
                event.round_no,
                if event.blocked { "ガードして狩" } else { "空振りして狩" },
                event.drop * 100.0
            ),
        }).collect(),
    })
}
