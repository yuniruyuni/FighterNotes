use super::common::is_biased;
use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};
use crate::match_events::{AdvantageOutcome, EventConfidence, MatchEvents};

/// ガードさせて有利を取ったのに攻めを継続せず、ターンを返した場面。
///
/// 有利のうちに動かないこと自体は、位置調整・ゲージ回復・様子見として
/// 正当な選択でありうる。このため「攻めなかった」だけでは指摘せず、
/// 続けて相手の攻撃を受ける側へ回った結果（`TurnLost`）を伴う場合だけ
/// 提示し、原因診断には偏りの共通条件を満たすことを要求する。
pub(crate) fn detect_advantage_abandoned(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let opportunities: Vec<_> = events
        .advantage_situations
        .iter()
        .filter(|event| event.side == own && event.confidence == EventConfidence::High)
        .collect();
    let abandoned: Vec<_> = opportunities
        .iter()
        .copied()
        .filter(|event| event.action_frame.is_none())
        .collect();
    let losses: Vec<_> = abandoned
        .iter()
        .copied()
        .filter(|event| event.outcome == AdvantageOutcome::TurnLost)
        .collect();
    if losses.is_empty() {
        return None;
    }
    let biased = is_biased(opportunities.len(), abandoned.len(), losses.len());
    let hp_lost: f32 = losses.iter().map(|event| event.drop).sum();
    let continued = opportunities.len() - abandoned.len();
    let abandoned_percent = abandoned.len() * 100 / opportunities.len();
    let average_plus = abandoned.iter().map(|event| event.plus_frames).sum::<u32>() as f32
        / abandoned.len() as f32;
    Some(AdviceCard {
        id: "advantage_abandoned".to_string(),
        kind: if biased {
            AdviceKind::Diagnosis
        } else {
            AdviceKind::Observation
        },
        confidence: EventConfidence::High,
        title: if biased {
            "ガードさせて有利を取った後に攻めを継続できていない"
        } else {
            "有利フレームを取った後にターンを渡した場面"
        }
        .to_string(),
        severity: hp_lost + 0.01 * losses.len() as f32,
        description: if biased {
            format!(
                "入力まで確認できた有利フレームの機会 {} 回中、{} 回（{}%）で次の攻撃を始めていません（平均 +{:.0}F）。うち {} 回はそのまま相手に攻め返され、合計 {:.0}% の HP を失いました（攻めを継続できたのは {} 回）。有利を取った直後に何もしない選択へ偏っており、相手に主導権を戻しています。",
                opportunities.len(),
                abandoned.len(),
                abandoned_percent,
                average_plus,
                losses.len(),
                hp_lost * 100.0,
                continued
            )
        } else {
            format!(
                "入力まで確認できた有利フレームの機会 {} 回中、{} 回（{}%）で次の攻撃を始めず、そのうち {} 回、合計 {:.0}% 被弾しています（平均 +{:.0}F、攻めを継続できたのは {} 回）。距離を取り直す・ゲージを回復する意図があった場合もあるため、この件数だけでは攻めの止まる癖とは{OBSERVATION_REVIEW_CAVEAT}。",
                opportunities.len(),
                abandoned.len(),
                abandoned_percent,
                losses.len(),
                hp_lost * 100.0,
                average_plus,
                continued
            )
        },
        practice: if biased {
            "該当クリップと同じ技をガードさせた状況をトレーニングで作り、そこから繋がる打撃と投げを1つずつ決めておきます。有利を確認したら必ずどちらかを出す、を先に体に入れてから選択肢を増やしましょう。"
        } else {
            "クリップで、有利を取った時点の距離とドライブゲージを確認します。離れていて届かない・回復を優先したなら問題ありません。密着で止まっている場合だけ、その状況からの攻め継続を1つ用意しましょう。"
        }
        .to_string(),
        evidence: losses
            .iter()
            .map(|event| EvidenceClip {
                frame: event.frame,
                end_frame: None,
                label: format!(
                    "R{} +{}Fで攻めを継続せず被弾 -{:.0}%",
                    event.round_no,
                    event.plus_frames,
                    event.drop * 100.0
                ),
            })
            .collect(),
    })
}
