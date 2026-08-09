use super::super::dir_arrow;
use crate::attack_info::AttackAttribute;
use crate::match_events::{DamageAttackEvidence, EventConfidence, GuardBreakEvent, MatchEvents};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};

fn attack_for_guard<'a>(
    events: &'a MatchEvents,
    guard: &GuardBreakEvent,
) -> Option<&'a DamageAttackEvidence> {
    let damage = events.damage.iter().find(|damage| {
        damage.victim == guard.side
            && damage.round_no == guard.round_no
            && damage.start_frame.abs_diff(guard.frame) <= 5
    })?;
    events
        .attack_evidence_for_damage(damage)
        .filter(|evidence| evidence.complete && evidence.confidence != EventConfidence::Low)
}

fn attribute_label(attribute: AttackAttribute) -> &'static str {
    match attribute {
        AttackAttribute::Upper => "上段",
        AttackAttribute::Middle => "中段",
        AttackAttribute::Lower => "下段",
        AttackAttribute::Throw => "投げ",
    }
}

/// ガード入力崩れ: ガード方向を握っていたのが途中で外れた被弾。
pub fn detect_guard_break(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let all_breaks: Vec<_> = events
        .guard_breaks
        .iter()
        .filter(|event| {
            event.side == own
                && attack_for_guard(events, event).and_then(|evidence| evidence.starter_attribute)
                    != Some(AttackAttribute::Throw)
        })
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
    let attributes: Vec<_> = all_breaks
        .iter()
        .filter_map(|event| {
            attack_for_guard(events, event).and_then(|evidence| evidence.starter_attribute)
        })
        .collect();
    let common_attribute = attributes.iter().copied().max_by_key(|candidate| {
        attributes
            .iter()
            .filter(|attribute| *attribute == candidate)
            .count()
    });
    let common_attribute_count = common_attribute
        .map(|candidate| {
            attributes
                .iter()
                .filter(|attribute| **attribute == candidate)
                .count()
        })
        .unwrap_or(0);
    let attribute_note = common_attribute.map_or_else(String::new, |attribute| {
        format!(
            " ゲーム内表示で攻撃属性まで確認できた {} 件のうち、{}が {} 件です。",
            attributes.len(),
            attribute_label(attribute),
            common_attribute_count
        )
    });
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
        hp_lost: Some(hp_lost),
        description: if repeated {
            format!(
                "ガード入力が外れた被弾を {} 回確認し、最も多い同一遷移は {}→{} の {} 回です。合計 {:.0}% 被弾しています。{}同じ方向への入力変更が複数回重なっているため、移動・ジャンプ・反撃を始めるタイミングの改善候補です。",
                all_breaks.len(), dir_arrow(&pattern.0), dir_arrow(&pattern.1), pattern_count, hp_lost * 100.0, attribute_note
            )
        } else {
            format!(
                "ガード中の入力が {}→{} に外れ、その非ガード状態で {:.0}% 被弾した場面が1回あります。{}この試合で同じ入力遷移による被弾は1回です。中下段や投げとの読み合いで意図的に動いた可能性もあるため、この1回だけで入力癖とは{OBSERVATION_REVIEW_CAVEAT}。",
                dir_arrow(&pattern.0), dir_arrow(&pattern.1), hp_lost * 100.0, attribute_note
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
            label: {
                let attack = attack_for_guard(events, event);
                let attribute = attack
                    .and_then(|evidence| evidence.starter_attribute)
                    .map(|attribute| format!("・{}", attribute_label(attribute)))
                    .unwrap_or_default();
                let damage = attack
                    .filter(|evidence| evidence.exact_damage_is_reliable())
                    .map(|evidence| format!("・{}ダメージ", evidence.combo_damage))
                    .unwrap_or_default();
                format!(
                    "R{} ガード入力崩れ {}→{} -{:.0}%{}{}",
                    event.round_no,
                    dir_arrow(&event.guard_dir),
                    dir_arrow(&event.broke_to),
                    event.drop * 100.0,
                    attribute,
                    damage
                )
            },
        }).collect(),
    })
}
