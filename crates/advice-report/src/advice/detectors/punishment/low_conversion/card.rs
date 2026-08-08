use super::model::LowReturn;
use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::EventConfidence;

pub(super) fn build(success_count: usize, lows: &[LowReturn]) -> AdviceCard {
    let repeated_input = lows
        .iter()
        .filter(|low| !low.input.is_empty())
        .map(|low| low.input.as_str())
        .max_by_key(|candidate| lows.iter().filter(|low| low.input == *candidate).count());
    let repeated_input_count = repeated_input
        .map(|input| lows.iter().filter(|low| low.input == input).count())
        .unwrap_or(0);
    let repeated = repeated_input_count >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let total_return: f32 = lows.iter().map(|low| low.drop).sum();
    let exact_values: Vec<_> = lows.iter().filter_map(|low| low.exact_damage).collect();
    let exact_note = if exact_values.is_empty() {
        String::new()
    } else {
        format!(
            " ゲーム内表示で確認できた累積ダメージは {} です。",
            exact_values
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(" / ")
        )
    };
    AdviceCard {
        id: "low_conversion".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::Medium,
        title: if repeated {
            "同じ確反入力が小さいリターンで終わっている"
        } else {
            "確反が小さいリターンで終わった場面"
        }.to_string(),
        severity: 0.03 * lows.len() as f32,
        // 損失は機会費用であり、この指摘が原因で失った HP ではない。
        hp_lost: None,
        description: if repeated {
            format!(
                "確反成功 {} 回中、12%未満の小さいリターンで終わった場面が {} 回、合計 {:.0}% あります。{}同じ入力 {} が {} 回含まれるため、ゲージ温存や位置取りを意図した選択でなければコンボへ繋ぐ改善候補です。",
                success_count, lows.len(), total_return * 100.0, exact_note, repeated_input.unwrap_or("?"), repeated_input_count
            )
        } else {
            format!(
                "確反成功 {} 回中、12%未満の小さいリターンで終わった場面が {} 回、合計 {:.0}% あります。{}ゲージ温存・位置・KO状況で単発止めが適切な場合もあるため、この件数だけではリターン不足の癖とは{OBSERVATION_REVIEW_CAVEAT}。この試合で同様の結果は {} 回です。",
                success_count, lows.len(), total_return * 100.0, exact_note, lows.len()
            )
        },
        practice: if repeated {
            "該当クリップと同じ始動から、画面中央・端それぞれの基本コンボを1つ用意します。ゲージを使わない構成から始め、繰り返し使っている単発反撃との差を確認しましょう。"
        } else {
            "クリップで残り体力・ゲージ・位置を確認します。意図的な温存なら問題ありません。伸ばせた場面なら、その始動から安定する短いコンボを1つだけ確認しましょう。"
        }.to_string(),
        evidence: lows.iter().map(|low| EvidenceClip {
            frame: low.frame,
            end_frame: None,
            label: format!(
                "R{} 確反{}が小リターン（-{:.0}%）{}",
                low.round_no,
                if low.input.is_empty() { String::new() } else { format!("（{}）", low.input) },
                low.drop * 100.0,
                low.exact_damage.map(|damage| {
                    let scaling = low
                        .final_scaling_percent
                        .map(|percent| format!("・最終{percent}%補正"))
                        .unwrap_or_default();
                    format!("・{damage}ダメージ{scaling}")
                }).unwrap_or_default()
            ),
        }).collect(),
    }
}
