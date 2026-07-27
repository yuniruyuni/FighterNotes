use super::super::dir_arrow;
use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{EventConfidence, MatchEvents};

/// ガード入力崩れ: ガード方向を握っていたのが途中で外れた被弾。
pub(crate) fn detect_guard_break(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let all_breaks: Vec<_> = events
        .guard_breaks
        .iter()
        .filter(|event| event.side == own)
        .collect();
    let pattern = all_breaks
        .iter()
        .map(|event| (event.guard_dir.clone(), event.broke_to.clone()))
        .max_by_key(|candidate| {
            all_breaks
                .iter()
                .filter(|event| event.guard_dir == candidate.0 && event.broke_to == candidate.1)
                .count()
        })?;
    let pattern_count = all_breaks
        .iter()
        .filter(|event| event.guard_dir == pattern.0 && event.broke_to == pattern.1)
        .count();
    let repeated = pattern_count >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = all_breaks.iter().map(|event| event.drop).sum();
    Some(AdviceCard {
        id: "guard_break".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if repeated {
            "同じ方向へガード入力が繰り返し崩れている"
        } else {
            "ガード入力が外れて被弾した場面"
        }
        .to_string(),
        severity: hp_lost,
        description: if repeated {
            format!(
                "ガード入力が外れた被弾を {} 回確認し、最も多い同一遷移は {}→{} の {} 回です。合計 {:.0}% 被弾しています。同じ方向への入力変更が複数回重なっているため、移動・ジャンプ・反撃を始めるタイミングの改善候補です。",
                all_breaks.len(), dir_arrow(&pattern.0), dir_arrow(&pattern.1), pattern_count, hp_lost * 100.0
            )
        } else {
            format!(
                "ガード中の入力が {}→{} に外れ、その非ガード状態で {:.0}% 被弾した場面が1回あります。この試合で同じ入力遷移による被弾は1回です。中下段や投げとの読み合いで意図的に動いた可能性もあるため、この1回だけで入力癖とは{OBSERVATION_REVIEW_CAVEAT}。",
                dir_arrow(&pattern.0), dir_arrow(&pattern.1), hp_lost * 100.0
            )
        },
        practice: if repeated {
            "相手の固めを記録し、ガード方向を握り続けたまま受け切る練習をします。反撃・移動・ジャンプを始める箇所を1つずつ確認し、ガード成立前に同じ方向へ動かないようにします。"
        } else {
            "クリップで、投げ・中下段を読んで意図的に動いたのか、反撃や移動を早く始めたのかを確認します。普段も同じ方向へ外している場合だけ、ガードを離すタイミングを遅らせましょう。"
        }.to_string(),
        evidence: all_breaks.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!("R{} ガード入力崩れ {}→{} -{:.0}%", event.round_no, dir_arrow(&event.guard_dir), dir_arrow(&event.broke_to), event.drop * 100.0),
        }).collect(),
    })
}
