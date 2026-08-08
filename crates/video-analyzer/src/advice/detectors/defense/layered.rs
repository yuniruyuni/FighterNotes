use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{
    DefenseResponseKind, EventConfidence, MatchEvents, ThreatOutcome, THREAT_DAMAGE_WINDOW,
};

/// Projectile plus teleport/cross-up pressure.
pub(crate) fn detect_layered_defense(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let threats: Vec<_> = events
        .compound_threats
        .iter()
        .filter(|threat| {
            threat.defender == own
                && threat.damage > 0.0
                && threat.outcome == ThreatOutcome::Hit
                && threat.projectile_response.as_ref().is_some_and(|response| {
                    response.kind == DefenseResponseKind::Parry
                        && response.end_frame < threat.followup_attack_frame
                })
                && threat.followup_response.is_none()
        })
        .collect();
    if threats.is_empty() {
        return None;
    }
    let opportunities = events
        .compound_threats
        .iter()
        .filter(|threat| threat.defender == own)
        .count();
    let repeated = threats.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = threats.iter().map(|threat| threat.damage).sum();
    Some(AdviceCard {
        id: "layered_defense".to_string(),
        kind: if repeated {
            AdviceKind::Diagnosis
        } else {
            AdviceKind::Observation
        },
        confidence: EventConfidence::High,
        title: if repeated {
            "複合攻撃への短いパリィが繰り返されている"
        } else {
            "複合攻撃でパリィ後に被弾した場面"
        }
        .to_string(),
        severity: hp_lost + 0.02 * threats.len() as f32,
        hp_lost: Some(hp_lost),
        description: if repeated {
            format!(
                "飛び道具とテレポート後の攻撃が重なる状況を {} 回確認し、そのうちパリィを後段まで維持できず被弾した場面が {} 回、合計 {:.0}% あります。同じ短いパリィで複数回被弾しているため、防御回答の見直し候補です。地上テレポートから投げも成立する状況ではパリィ固定が正解とは限りません。",
                opportunities,
                threats.len(),
                hp_lost * 100.0
            )
        } else {
            format!(
                "飛び道具とテレポート後の攻撃が重なる状況で、パリィを入力したものの後段まで維持できず {:.0}% 被弾した場面が 1 回あります。この1回だけでは、投げを警戒して離した読み合いの結果か、短く離す傾向があるかは{OBSERVATION_REVIEW_CAVEAT}。この試合で確認できた同様の被弾は1回です。",
                hp_lost * 100.0
            )
        },
        practice: if repeated {
            "トレーニングモードで「飛び道具→テレポート攻撃」を記録し、飛び道具を受けた時点でパリィを離さず、後ろからの打撃まで受け切る練習をします。投げも成立する状況ではガード・無敵技を含めて回答を散らします。"
        } else {
            "クリップでパリィを離した理由を確認します。後段の打撃を見落としていた場合は長めのパリィを試し、投げを警戒した判断なら他の同状況でも同じ離し方に偏っていないかを見比べましょう。"
        }
        .to_string(),
        evidence: threats
            .iter()
            .map(|threat| EvidenceClip {
                frame: threat.projectile_start_frame,
                end_frame: Some(
                    threat
                        .followup_attack_frame
                        .saturating_add(THREAT_DAMAGE_WINDOW),
                ),
                label: format!("R{} 飛び道具＋テレポート連係", threat.round_no),
            })
            .collect(),
    })
}
