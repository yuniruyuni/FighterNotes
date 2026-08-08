use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{EventConfidence, MatchEvents, WhiffOutcome};

/// 届かなかった技の硬直を狩られている場面。
///
/// 技を空振りすること自体は差し合いの一部で、間合いを測る目的の空振りも
/// ある。このため空振り回数そのものは指摘せず、実際に狩られた場面だけを
/// 提示する。単発は読み負けと区別できないため反復を求める。
///
/// 投げ・Drive Impact・無敵技は専用カードが結果を扱うため含まない。
pub(crate) fn detect_whiff_punished(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let whiffs: Vec<_> = events
        .whiffs
        .iter()
        .filter(|whiff| whiff.side == own && whiff.confidence == EventConfidence::High)
        .collect();
    let punished: Vec<_> = whiffs
        .iter()
        .copied()
        .filter(|whiff| whiff.outcome == WhiffOutcome::Punished)
        .collect();
    if punished.is_empty() {
        return None;
    }
    let repeated = punished.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = punished.iter().map(|whiff| whiff.drop).sum();
    let punished_percent = punished.len() * 100 / whiffs.len();
    Some(AdviceCard {
        id: "whiff_punished".to_string(),
        kind: if repeated {
            AdviceKind::Diagnosis
        } else {
            AdviceKind::Observation
        },
        confidence: EventConfidence::High,
        title: if repeated {
            "届かない技の硬直を繰り返し狩られている"
        } else {
            "空振りした技の硬直を狩られた場面"
        }
        .to_string(),
        severity: hp_lost + 0.01 * punished.len() as f32,
        hp_lost: Some(hp_lost),
        description: if repeated {
            format!(
                "接触しなかった技 {} 回のうち、{} 回（{}%）で硬直を狩られ、合計 {:.0}% の HP を失いました。空振り自体は間合いを測る手段として正当ですが、届かない位置から出した技を複数回反撃されているため、技を置く距離とタイミングが改善対象です。",
                whiffs.len(),
                punished.len(),
                punished_percent,
                hp_lost * 100.0
            )
        } else {
            format!(
                "接触しなかった技 {} 回のうち、{} 回（{}%）で硬直を狩られ、合計 {:.0}% の HP を失いました。間合いを測る空振りは差し合いの一部なので、この件数だけでは技を置く距離の癖とは{OBSERVATION_REVIEW_CAVEAT}。",
                whiffs.len(),
                punished.len(),
                punished_percent,
                hp_lost * 100.0
            )
        },
        practice: if repeated {
            "クリップで、相手のどの位置に対して技を出していたかを確認します。トレーニングで相手を歩かせ、自分の主力技が届く距離と届かない距離の境目を体で覚えてから、その手前で振る練習をしましょう。"
        } else {
            "クリップで、間合いを測る意図の空振りだったか、届くつもりで外したかを確認します。届くつもりだった場合だけ、その技の実際のリーチを確認しましょう。"
        }
        .to_string(),
        evidence: punished
            .iter()
            .map(|whiff| EvidenceClip {
                frame: whiff.frame,
                end_frame: Some(whiff.end_frame),
                label: format!(
                    "R{} 空振りの硬直を狩られた -{:.0}%",
                    whiff.round_no,
                    whiff.drop * 100.0
                ),
            })
            .collect(),
    })
}
