use crate::match_events::{BurnoutCause, EventConfidence, MatchEvents};
use crate::{AdviceCard, AdviceKind, EvidenceClip};

pub fn detect_burnout(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let periods: Vec<_> = events
        .burnouts
        .iter()
        .filter(|period| period.side == own)
        .collect();
    if periods.is_empty() {
        return None;
    }
    let total_sec: f32 = periods
        .iter()
        .map(|period| (period.end_frame - period.start_frame) as f32 / 60.0)
        .sum();
    let hp_lost: f32 = periods.iter().map(|period| period.hp_lost).sum();
    let hp_dealt: f32 = periods.iter().map(|period| period.hp_dealt).sum();
    let count = |cause| {
        periods
            .iter()
            .filter(|period| period.cause == cause)
            .count()
    };
    let self_spent = count(BurnoutCause::SelfInitiated);
    let forced = count(BurnoutCause::ForcedByGuard);
    let mixed = count(BurnoutCause::Mixed);
    let unknown = periods.len() - self_spent - forced - mixed;
    let rounds: Vec<u32> = {
        let mut values: Vec<u32> = periods.iter().map(|period| period.round_no).collect();
        values.dedup();
        values
    };
    let duration = if total_sec >= 1.0 {
        format!("合計 {total_sec:.0} 秒間をゲージなしで戦い")
    } else {
        "ラウンド終了までゲージなしのまま".to_string()
    };
    let causes = format!(
        "突入直前は自分のゲージ使用 {} 回、ガードで削られた場面 {} 回、両方 {} 回、分類保留 {} 回です",
        self_spent, forced, mixed, unknown
    );
    Some(AdviceCard {
        id: "burnout".to_string(),
        kind: AdviceKind::Statistic,
        confidence: if periods.iter().all(|period| period.confidence == EventConfidence::High) {
            EventConfidence::High
        } else {
            EventConfidence::Medium
        },
        title: "バーンアウト管理".to_string(),
        severity: hp_lost + 0.03 * periods.len() as f32,
        hp_lost: Some(hp_lost),
        description: format!(
            "バーンアウトに {} 回入り、{}、その間の被ダメは {:.0}%、与ダメは {:.0}% でした（ラウンド {}）。{}。被ダメだけでなく、攻めのために使い切ったのか、守りで削り切られたのかを分けて見直しましょう。",
            periods.len(), duration, hp_lost * 100.0, hp_dealt * 100.0,
            rounds.iter().map(u32::to_string).collect::<Vec<_>>().join(", "), causes
        ),
        practice: "各クリップの直前 10 秒を見て、攻めを継続するための消費だったか、ガードで使わされたかを分類しましょう。不要だった消費を 1 つだけ減らす方針にすると再現しやすくなります。".to_string(),
        evidence: periods.iter().map(|period| EvidenceClip {
            frame: period.start_frame,
            end_frame: None,
            label: format!("R{} バーンアウト", period.round_no),
        }).collect(),
    })
}
