use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, BIG_HIT_LIST};
use crate::attack_info::AttackAttribute;
use crate::match_events::{
    AttackDamageConsistency, DamageAttackEvidence, EventConfidence, MatchEvents, JUMP_ATTACK_MAX,
    JUMP_SELF_HIT_WINDOW,
};

fn attribute_label(attribute: AttackAttribute) -> &'static str {
    match attribute {
        AttackAttribute::Upper => "上段",
        AttackAttribute::Middle => "中段",
        AttackAttribute::Lower => "下段",
        AttackAttribute::Throw => "投げ",
    }
}

fn evidence_label(
    round_no: u32,
    drop: f32,
    hp_after: f32,
    attack: Option<&DamageAttackEvidence>,
) -> String {
    let base = format!(
        "R{} -{:.0}% 被弾（残り HP {:.0}%）",
        round_no,
        drop * 100.0,
        hp_after * 100.0
    );
    let Some(attack) = attack else {
        return base;
    };
    if attack.hp_consistency == AttackDamageConsistency::Mismatch {
        return format!(
            "{base} / ゲーム内表示 {}（HPと不一致）",
            attack.combo_damage
        );
    }
    if !attack.exact_damage_is_reliable() {
        return base;
    }
    let starter = attack
        .starter_attribute
        .map(|attribute| format!("・{}始動", attribute_label(attribute)))
        .unwrap_or_default();
    let sequence = if attack.sequence_count > 1 {
        format!("・{}連係合計", attack.sequence_count)
    } else {
        String::new()
    };
    format!(
        "{base} / {}ダメージ{starter}{sequence}・最終{}%補正",
        attack.combo_damage, attack.final_scaling_percent
    )
}

fn evidence_window(card_id: &str, evidence: &EvidenceClip) -> Option<(u32, u32)> {
    match card_id {
        "layered_defense"
        | "mashing"
        | "committed_button_vs_di"
        | "teleport_defense"
        | "reversal_punished"
        | "punish_fail" => Some((
            evidence.frame,
            evidence
                .end_frame
                .unwrap_or(evidence.frame.saturating_add(180)),
        )),
        "anti_air" => Some((
            evidence.frame,
            evidence.frame.saturating_add(JUMP_ATTACK_MAX + 60),
        )),
        "own_jumps" => Some((
            evidence.frame,
            evidence.frame.saturating_add(JUMP_SELF_HIT_WINDOW + 60),
        )),
        "press_while_minus" | "throw_while_minus" => {
            Some((evidence.frame, evidence.frame.saturating_add(60)))
        }
        "guard_break" => Some((evidence.frame.saturating_sub(10), evidence.frame + 30)),
        "throw_loop" => Some((evidence.frame, evidence.frame.saturating_add(120))),
        "throw_whiff_punished" | "throw_interrupted_by_invincible" => evidence
            .end_frame
            .map(|end_frame| (evidence.frame, end_frame)),
        _ => None,
    }
}

pub(crate) fn detect_big_hits(
    events: &MatchEvents,
    own: u8,
    existing_cards: &[AdviceCard],
) -> Option<AdviceCard> {
    let hits: Vec<_> = events
        .damage
        .iter()
        .filter(|damage| {
            damage.victim == own
                && damage.drop >= BIG_HIT_LIST
                && !existing_cards.iter().any(|card| {
                    card.evidence.iter().any(|evidence| {
                        evidence_window(&card.id, evidence).is_some_and(|(start, end)| {
                            damage.start_frame <= end && damage.end_frame >= start
                        })
                    })
                })
        })
        .collect();
    if hits.is_empty() {
        return None;
    }
    let hp_lost: f32 = hits.iter().map(|damage| damage.drop).sum();
    let exact_count = hits
        .iter()
        .filter_map(|damage| events.attack_evidence_for_damage(damage))
        .filter(|evidence| evidence.exact_damage_is_reliable())
        .count();
    let mismatch_count = hits
        .iter()
        .filter_map(|damage| events.attack_evidence_for_damage(damage))
        .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Mismatch)
        .count();
    let attack_note = if exact_count > 0 || mismatch_count > 0 {
        format!(
            " うち {exact_count} 件はゲーム内表示の累積ダメージ・補正率まで確認でき、HP表示との不一致は {mismatch_count} 件です。"
        )
    } else {
        String::new()
    };
    Some(AdviceCard {
        id: "big_hits".to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::High,
        title: "原因を分類できなかった大ダメージ".to_string(),
        severity: hp_lost,
        description: format!(
            "一度のコンボ・連係で HP を {:.0}% 以上失い、対空・暴れ・不利後の回答など既存の原因別カードへ分類できなかった被弾が {} 回、合計 {:.0}% あります。{}各場面で被弾直前の行動を確認し、同じ入り口が繰り返されているかを利用者が判断するための一覧です。",
            BIG_HIT_LIST * 100.0, hits.len(), hp_lost * 100.0, attack_note
        ),
        practice: "各場面を再生し、被弾直前の行動を「差し返された／DI対応漏れ／投げ／中下段／その他」でメモします。同じ分類が複数回ある場合だけ、次に詳しく調べる改善候補にしましょう。".to_string(),
        evidence: hits.iter().map(|damage| EvidenceClip {
            frame: damage.pre_freeze_frame,
            end_frame: Some(damage.end_frame),
            label: evidence_label(
                damage.round_no,
                damage.drop,
                damage.hp_after,
                events.attack_evidence_for_damage(damage),
            ),
        }).collect(),
    })
}
