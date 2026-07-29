use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{
    DamageEvent, EventConfidence, MatchEvents, ThrowActionEvent, ThrowOutcome,
};

/// 投げの空振りから次の被弾までを見直す最大範囲（動画フレーム）。
///
/// 連続して2回投げを空振った後に反撃された場合も、同じ一連の場面として
/// まとめられる長さにする。因果関係までは断定せず、クリップ確認用に使う。
const THROW_WHIFF_DAMAGE_WINDOW: u32 = 90;

struct PunishedWhiffGroup<'a> {
    damage: &'a DamageEvent,
    whiffs: Vec<&'a ThrowActionEvent>,
}

/// 実行まで確定した自分の投げ空振り後、短時間内に被弾した場面を提示する。
pub(crate) fn detect_throw_whiff_punished(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let mut groups: Vec<PunishedWhiffGroup<'_>> = Vec::new();
    for whiff in events.throw_actions.iter().filter(|event| {
        event.thrower == own
            && event.outcome == ThrowOutcome::ExecutedWhiff
            && event.confidence == EventConfidence::High
    }) {
        let anchor = whiff.active_frame.unwrap_or(whiff.input_frame);
        let Some(damage) = events
            .damage
            .iter()
            .filter(|damage| {
                damage.victim == own
                    && damage.round_no == whiff.round_no
                    && damage.start_frame >= anchor
                    && damage.start_frame <= anchor.saturating_add(THROW_WHIFF_DAMAGE_WINDOW)
            })
            .min_by_key(|damage| damage.start_frame)
        else {
            continue;
        };
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.damage.start_frame == damage.start_frame)
        {
            group.whiffs.push(whiff);
        } else {
            groups.push(PunishedWhiffGroup {
                damage,
                whiffs: vec![whiff],
            });
        }
    }
    if groups.is_empty() {
        return None;
    }

    groups.sort_by_key(|group| group.whiffs[0].input_frame);
    let whiff_count: usize = groups.iter().map(|group| group.whiffs.len()).sum();
    let hp_lost: f32 = groups.iter().map(|group| group.damage.drop).sum();
    let repeated = whiff_count >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    Some(AdviceCard {
        id: "throw_whiff_punished".to_string(),
        kind: if repeated {
            AdviceKind::Diagnosis
        } else {
            AdviceKind::Observation
        },
        confidence: EventConfidence::High,
        title: if repeated {
            "投げ空振りを繰り返して反撃を受けている"
        } else {
            "投げ空振り後に被弾した場面"
        }
        .to_string(),
        severity: hp_lost + 0.02 * whiff_count as f32,
        description: if repeated {
            format!(
                "実行まで確認できた投げ空振りが {} 回あり、その後約1.5秒以内に被弾した一連の場面が {} 件、合計 {:.0}% あります。連続した空振りは同じ被弾へまとめています。相手の後退や無敵を読んだ結果かまでは断定しませんが、複数回確認できたため投げを押す距離・タイミングを見直す候補です。",
                whiff_count,
                groups.len(),
                hp_lost * 100.0
            )
        } else {
            format!(
                "実行まで確認できた投げ空振りの後、約1.5秒以内に {:.0}% 被弾した場面が1件あります。この1回だけでは、相手の後退を読めなかったのか、別の読み合いが続いた結果かは{OBSERVATION_REVIEW_CAVEAT}。",
                hp_lost * 100.0
            )
        },
        practice: if repeated {
            "クリップで、相手が投げ間合いにいたか、後退や垂直ジャンプを見ずに投げを連打していないかを確認します。投げ間合い外では歩きガードに戻し、投げを空振った後は同じ入力を重ねない練習をします。"
        } else {
            "クリップをスロー再生し、投げ入力時の距離と相手の後退開始を確認します。意図した読み負けなら単発の失敗として扱い、普段も同じ距離で投げている場合だけ入力を遅らせましょう。"
        }
        .to_string(),
        evidence: groups
            .iter()
            .map(|group| EvidenceClip {
                frame: group.whiffs[0].input_frame,
                end_frame: Some(group.damage.end_frame),
                label: format!(
                    "R{} 投げ空振り{}後に被弾 -{:.0}%",
                    group.damage.round_no,
                    if group.whiffs.len() > 1 {
                        format!("{}回", group.whiffs.len())
                    } else {
                        String::new()
                    },
                    group.damage.drop * 100.0
                ),
            })
            .collect(),
    })
}
