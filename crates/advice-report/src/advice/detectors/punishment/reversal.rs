use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MIN_REPEATED_NEGATIVE_OUTCOMES, OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{EventConfidence, MatchEvents};

pub(crate) fn detect_reversal_punished(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let supers: Vec<_> = events
        .super_arts
        .iter()
        .filter(|event| {
            event.side == own && event.punished && event.confidence == EventConfidence::High
        })
        .collect();
    let reversals: Vec<_> = events
        .reversals
        .iter()
        .filter(|event| {
            event.side == own
                && event.confidence == EventConfidence::High
                && !supers.iter().any(|super_art| {
                    super_art.round_no == event.round_no
                        && super_art.frame.abs_diff(event.frame) <= 120
                })
        })
        .collect();
    if reversals.is_empty() && supers.is_empty() {
        return None;
    }
    let occurrence_count = reversals.len() + supers.len();
    let repeated = occurrence_count >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = reversals.iter().map(|event| event.drop).sum::<f32>()
        + supers
            .iter()
            .map(|event| event.punished_damage)
            .sum::<f32>();
    let super_only = reversals.is_empty();
    let mut evidence: Vec<_> = supers
        .iter()
        .map(|event| EvidenceClip {
            frame: event.frame,
            end_frame: None,
            label: format!(
                "R{} {}後に反撃 -{:.0}%",
                event.round_no,
                super_label(event.level, event.critical_art),
                event.punished_damage * 100.0
            ),
        })
        .collect();
    evidence.extend(reversals.iter().map(|event| EvidenceClip {
        frame: event.frame,
        end_frame: None,
        label: format!(
            "R{} 無敵技を{}られ -{:.0}%",
            event.round_no,
            if event.blocked {
                "ガードして狩"
            } else {
                "空振りして狩"
            },
            event.drop * 100.0
        ),
    }));
    evidence.sort_by_key(|clip| clip.frame);
    Some(AdviceCard {
        id: "reversal_punished".to_string(),
        kind: if repeated { AdviceKind::Diagnosis } else { AdviceKind::Observation },
        confidence: EventConfidence::High,
        title: if super_only && repeated {
            "SA/CAの後隙を繰り返し反撃されている"
        } else if super_only {
            "SA/CAの後隙に反撃を受けた場面"
        } else if repeated {
            "無敵技という防御回答を繰り返し狩られている"
        } else {
            "無敵技を狩られた場面"
        }.to_string(),
        severity: hp_lost + 0.02 * occurrence_count as f32,
        hp_lost: Some(hp_lost),
        description: if super_only && repeated {
            format!(
                "ゲージ消費で確認したSA/CAが通らず、その後に反撃を受けた場面が {occurrence_count} 回、合計 {:.0}% あります。SA/CAという高コストの回答で複数回損失が出ているため、使用場面を見直す候補です。",
                hp_lost * 100.0
            )
        } else if super_only {
            format!(
                "ゲージ消費で確認した{}が通らず、その後に {:.0}% の反撃を受けた場面が1回あります。単発の読み負けだけでは選択自体の誤りとは断定できないため、この場面を{OBSERVATION_REVIEW_CAVEAT}。",
                super_label(supers[0].level, supers[0].critical_art),
                hp_lost * 100.0
            )
        } else if repeated {
            format!(
                "無敵技（昇竜・SAなど）が通らず、後隙を狩られた場面が {} 回、合計 {:.0}% あります。技名までは確定していませんが、無敵技という同じ防御回答で複数回損失が出ているため、選択頻度を見直す候補です。",
                occurrence_count, hp_lost * 100.0
            )
        } else {
            format!(
                "無敵技（昇竜・SAなど）が通らず、後隙を狩られて {:.0}% 被弾した場面が1回あります。無敵技は打撃重ねへの正しい回答にもなるため、この1回だけでは読み負けか選択の偏りかを{OBSERVATION_REVIEW_CAVEAT}。この試合で同様の被弾は1回です。",
                hp_lost * 100.0
            )
        },
        practice: if super_only && repeated {
            "各クリップで、コンボ確認・確定反撃・切り返し・単発のどの文脈だったかと、相手にガードされたか即時接触が無かったかを確認します。同じ文脈で反撃されている場合だけ、確認してからSAへつなぐ練習や切り返しの選択比率を調整しましょう。"
        } else if super_only {
            "クリップで、ヒット確認や確定反撃として撃ったのか、切り返し・単発だったのかを確認します。意図した読みなら単発の失敗として扱い、確認なしで撃っていた場合だけ同じ状況をトレーニングモードで再現しましょう。"
        } else if repeated {
            "起き攻めや固めを記録し、ガード・遅らせ投げ抜け・バックステップ・無敵技を混ぜます。無敵技を同じ局面で連続して選ばず、相手が打撃重ねに偏ったと確認したときだけ比率を上げましょう。"
        } else {
            "クリップで、相手の打撃重ねを読んで撃ったのか、苦しくなって無意識に撃ったのかを確認します。意図した読みなら単発の失敗として扱い、普段も同じ局面で撃っている場合だけ頻度を下げましょう。"
        }.to_string(),
        evidence,
    })
}

fn super_label(level: u8, critical_art: bool) -> &'static str {
    if critical_art {
        "CA"
    } else {
        match level {
            1 => "SA1",
            2 => "SA2",
            _ => "SA3",
        }
    }
}
