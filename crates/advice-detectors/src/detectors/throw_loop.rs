use crate::match_events::{EventConfidence, MatchEvents, ThrowEvent};
use crate::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT, THROW_STREAK_MIN};

/// 相手の投げが連続で通っているかを検出する。
pub fn detect_throw_loop(events: &MatchEvents, opponent: u8) -> Option<AdviceCard> {
    let attempts: Vec<_> = events
        .throws
        .iter()
        .filter(|event| event.thrower == opponent)
        .collect();
    let connected: Vec<_> = attempts
        .iter()
        .copied()
        .filter(|event| event.connected)
        .collect();
    if connected.is_empty() {
        return None;
    }
    let mut best_streak: Vec<&ThrowEvent> = Vec::new();
    let mut current: Vec<&ThrowEvent> = Vec::new();
    for event in &attempts {
        if event.connected {
            if current
                .last()
                .is_some_and(|last| event.frame > last.frame + 900)
            {
                current.clear();
            }
            current.push(event);
            if current.len() > best_streak.len() {
                best_streak = current.clone();
            }
        } else {
            current.clear();
        }
    }
    let repeated = best_streak.len() as u32 >= THROW_STREAK_MIN;
    Some(AdviceCard {
        id: "throw_loop".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if repeated { "投げを連続して受けている" } else { "投げを受けた場面" }.to_string(),
        severity: 0.12 * connected.len() as f32,
        // ThrowEvent は被ダメージを持たない。推定で埋めず未設定にする。
        hp_lost: None,
        description: if repeated {
            format!(
                "相手の投げ {} 回中 {} 回が通り、最大 {} 回連続で投げられています。時々投げられるのは正常な読み合いですが、3回以上連続したため同じ守り方が続いていないか見直す候補です。",
                attempts.len(), connected.len(), best_streak.len()
            )
        } else {
            format!(
                "相手の投げ {} 回中 {} 回が通り、この試合の最大連続被投げは {} 回です。投げは打撃との読み合いなので、この結果だけで守り方が悪いとは{OBSERVATION_REVIEW_CAVEAT}。同様に投げられた全 {} 場面を確認できます。",
                attempts.len(), connected.len(), best_streak.len(), connected.len()
            )
        },
        practice: if repeated {
            "打撃重ねと投げをランダム再生し、遅らせ投げ抜け・バックステップ・垂直ジャンプを混ぜます。同じ防御回答を連続して選ばず、まず3連続で投げられないことを目標にしましょう。"
        } else {
            "クリップで、打撃を警戒してガードを選んだ結果か、毎回同じ守り方をしていたかを確認します。意図した読み負けなら問題ありません。"
        }.to_string(),
        evidence: connected.iter().map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!("R{} 投げられた", event.round_no),
        }).collect(),
    })
}
