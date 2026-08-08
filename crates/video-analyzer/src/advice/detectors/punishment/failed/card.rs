use super::analysis::Summary;
use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};
use crate::match_events::EventConfidence;

pub(super) fn build(summary: &Summary<'_>, option_text: &str) -> AdviceCard {
    AdviceCard {
        id: "punish_fail".to_string(),
        kind: if summary.repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if summary.repeated {
            "同じ反撃入力が繰り返し届いていない"
        } else {
            "ガード後の反撃が届かなかった場面"
        }.to_string(),
        severity: summary.hp_lost + 0.03 * summary.failures.len() as f32,
        hp_lost: Some(summary.hp_lost),
        description: if summary.repeated {
            format!(
                "相手の技をガードし、反撃が間に合う近〜中距離で攻撃を出したものの届かなかった場面が {} 回あります。同じ入力 {} が {} 回含まれ、確反成功は {} 回、空振り後の被ダメは合計 {:.0}% です。同じ距離で届かない反撃を繰り返している可能性があるため、技選択を見直す候補です。{}",
                summary.failures.len(), summary.repeated_input.unwrap_or("?"), summary.repeated_input_count,
                summary.success_count, summary.hp_lost * 100.0, option_text
            )
        } else {
            format!(
                "相手の技をガードした後、反撃が間に合う近〜中距離で攻撃を出したものの届かなかった場面が {} 回あります（確反成功は {} 回）。空振り後の被ダメは合計 {:.0}% です。まずその距離で確定する技があるかを確認してください。距離調整や先端ガードによって適切な技が変わるため、この件数だけで反撃の癖とは{OBSERVATION_REVIEW_CAVEAT}。{}",
                summary.failures.len(), summary.success_count, summary.hp_lost * 100.0, option_text
            )
        },
        practice: if summary.repeated {
            "相手の後隙が大きい技を実戦と同じ距離で記録し、密着と先端で使う反撃を分けます。繰り返し届かなかった入力の代わりに、確実に届く技を10回連続で決めましょう。"
        } else {
            "クリップと同じ距離をトレーニングモードで再現し、実際に確定して届く技を確認します。距離依存の単発空振りなら、癖ではなく場面固有の技選択として扱います。"
        }.to_string(),
        evidence: summary.failures.iter().map(|punish| {
            let tail = if punish.punished_drop > 0.0 {
                format!(" → -{:.0}% 被弾", punish.punished_drop * 100.0)
            } else {
                String::new()
            };
            let pressed = if punish.pressed.is_empty() {
                String::new()
            } else {
                format!("（{}）", punish.pressed)
            };
            EvidenceClip {
                frame: punish.frame,
                end_frame: None,
                label: format!(
                    "R{} ガード後の反撃空振り +{}F / 距離確認{}{}",
                    punish.round_no, punish.advantage, pressed, tail
                ),
            }
        }).collect(),
    }
}
