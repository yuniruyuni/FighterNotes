use crate::match_events::{
    EventConfidence, JumpDirection, JumpOutcome, MatchEvents, JUMP_SELF_HIT_MIN,
    JUMP_SELF_HIT_WINDOW,
};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};

/// 自分の飛びが落とされているのに飛び続けていないかを検出する。
pub fn detect_own_jumps(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let own_jumps: Vec<_> = events
        .jumps
        .iter()
        .filter(|jump| {
            jump.side == own && jump.takeoff_confirmed && jump.direction != JumpDirection::Backward
        })
        .collect();
    let got_hit: Vec<_> = own_jumps
        .iter()
        .filter(|jump| jump.outcome == JumpOutcome::GotHit)
        .collect();
    if got_hit.is_empty() {
        return None;
    }
    let successful = own_jumps
        .iter()
        .filter(|jump| jump.outcome == JumpOutcome::LandedHit)
        .count();
    let resolved = got_hit.len() + successful;
    let repeated =
        got_hit.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES && got_hit.len() * 100 >= resolved * 50;
    let kind = if repeated {
        AdviceKind::Diagnosis
    } else {
        AdviceKind::Observation
    };
    let hp_lost: f32 = got_hit
        .iter()
        .filter_map(|jump| {
            events.damage.iter().find(|damage| {
                let contact_matches = jump
                    .contact_frame
                    .is_some_and(|contact| damage.start_frame.abs_diff(contact) <= 25);
                damage.victim == own
                    && (contact_matches
                        || (jump.contact_frame.is_none()
                            && damage.start_frame >= jump.frame + JUMP_SELF_HIT_MIN
                            && damage.start_frame
                                <= jump.air_end.max(jump.frame + JUMP_SELF_HIT_WINDOW)))
            })
        })
        .map(|damage| damage.drop)
        .sum();
    Some(AdviceCard {
        id: "own_jumps".to_string(),
        kind,
        confidence: EventConfidence::High,
        title: match kind {
            AdviceKind::Diagnosis => "ジャンプを繰り返し落とされている",
            _ => "ジャンプを落とされた場面",
        }.to_string(),
        severity: hp_lost + 0.02 * got_hit.len() as f32,
        hp_lost: Some(hp_lost),
        description: match kind {
            AdviceKind::Diagnosis => format!(
                "自分の前・垂直ジャンプ {} 回のうち {} 回が迎撃され、合計 {:.0}% の HP を失いました。攻防結果を確定できたジャンプの半数以上を複数回落とされているため、相手が対空を見せた後も同じ接近手段を選んでいないか見直す価値があります。",
                own_jumps.len(), got_hit.len(), hp_lost * 100.0
            ),
            _ => format!(
                "自分の前・垂直ジャンプ {} 回のうち {} 回が迎撃され、合計 {:.0}% の HP を失いました。この試合で同様の被対空は {} 回です。単発の読み負けか、ジャンプへ偏った結果かはこの件数だけでは{OBSERVATION_REVIEW_CAVEAT}。",
                own_jumps.len(), got_hit.len(), hp_lost * 100.0, got_hit.len()
            ),
        },
        practice: match kind {
            AdviceKind::Diagnosis => "落とされたジャンプの直前状況を確認し、接近手段をジャンプ以外に2つ（歩きガード・ドライブラッシュ等）用意します。相手が対空を見せた後は、同じ接近を連続で選ばない練習が有効です。",
            _ => "クリップでジャンプを選んだ理由を確認します。相手の対空を試した1回なら問題ありませんが、普段も同じ距離・同じタイミングで飛んでいる場合は接近手段を散らしましょう。",
        }.to_string(),
        evidence: got_hit.iter().map(|jump| EvidenceClip {
            frame: jump.frame,
            end_frame: None,
            label: format!("R{} ジャンプを迎撃された", jump.round_no),
        }).collect(),
    })
}
