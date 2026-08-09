use crate::match_events::{DamageEvent, EventConfidence, MatchEvents};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, RoundSummary, EARLY_HIT_FRAMES, OBSERVATION_REVIEW_CAVEAT,
};

pub fn detect_early_hits(
    events: &MatchEvents,
    rounds: &[RoundSummary],
    own: u8,
) -> Option<AdviceCard> {
    let early: Vec<_> = rounds.iter().filter(|round| round.early_hit).collect();
    if early.is_empty() {
        return None;
    }
    let first_hit = |round: &RoundSummary| -> Option<&DamageEvent> {
        events.damage.iter().find(|damage| {
            damage.round_no == round.round_no
                && damage.victim == own
                && damage.start_frame < round.start_frame + EARLY_HIT_FRAMES
        })
    };
    Some(AdviceCard {
        id: "early_hits".to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::Medium,
        title: "開幕に被弾したラウンド".to_string(),
        severity: 0.05 * early.len() as f32,
        hp_lost: Some(
            early
                .iter()
                .filter_map(|round| first_hit(round))
                .map(|damage| damage.drop)
                .sum(),
        ),
        description: format!(
            "{} ラウンド中 {} ラウンドで開幕 3 秒以内に被弾しています。この試合で同様の開幕被弾は {} 回です。現時点では最初に選んだ行動が同じかまでは確認できないため、回数が多くても開幕行動の癖とは{OBSERVATION_REVIEW_CAVEAT}。各クリップで最初の入力が共通しているかを確認してください。",
            rounds.len(), early.len(), early.len()
        ),
        practice: "各ラウンドの最初の行動をメモし、同じ技・ジャンプ・前進が続いていた場合だけ、次の対戦で様子見ガードを混ぜて選択率を散らしましょう。".to_string(),
        evidence: early.iter().map(|round| match first_hit(round) {
            Some(damage) => EvidenceClip {
                frame: damage.pre_freeze_frame,
                end_frame: Some(damage.end_frame),
                label: format!("R{} 開幕被弾 -{:.0}%", round.round_no, damage.drop * 100.0),
            },
            None => EvidenceClip {
                frame: round.start_frame,
                end_frame: None,
                label: format!("R{} 開幕被弾", round.round_no),
            },
        }).collect(),
    })
}
