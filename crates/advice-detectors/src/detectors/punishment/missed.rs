use super::super::move_identity::identify_contact_move;
use super::options::missed_option_text;
use crate::match_events::{EventConfidence, MatchEvents, PunishOutcome, PunishReachability};
use crate::{AdviceCard, AdviceKind, EvidenceClip};

pub fn detect_punish_missed(
    events: &MatchEvents,
    own: u8,
    own_character: Option<&str>,
    opponent_character: Option<&str>,
) -> Option<AdviceCard> {
    let opponent = 3 - own;
    let missed: Vec<_> = events
        .punishes
        .iter()
        .filter(|punish| {
            punish.side == own
                && punish.outcome == PunishOutcome::Missed
                && punish.reachability == PunishReachability::Confirmed
        })
        .collect();
    if missed.is_empty() {
        // 距離を確認できた見逃しが無くても、反撃猶予そのものは実測できて
        // いる場面が繰り返しあれば、断定しない観察としてシーンを示す。
        return detect_unconfirmed_misses(events, own, own_character);
    }
    // ガードした技の同定。一意に絞れた場面だけ技名を持つ。
    let identified: Vec<Option<&crate::frame_data::MoveData>> = missed
        .iter()
        .map(|punish| {
            punish.source_contact_frame.and_then(|contact| {
                identify_contact_move(events, opponent, opponent_character, contact)
            })
        })
        .collect();
    let min_advantage = missed
        .iter()
        .map(|punish| punish.advantage)
        .min()
        .unwrap_or(0);
    let option_text = missed_option_text(own_character, min_advantage);
    // 同じ技を繰り返し見逃していれば、覚えるべき技として名指しする。
    let repeated_move = identified
        .iter()
        .flatten()
        .map(|move_data| move_data.name.as_str())
        .max_by_key(|name| {
            identified
                .iter()
                .flatten()
                .filter(|move_data| move_data.name == *name)
                .count()
        })
        .and_then(|name| {
            let count = identified
                .iter()
                .flatten()
                .filter(|move_data| move_data.name == name)
                .count();
            (count >= 2).then_some((name, count))
        });
    // 技を同定できた時点で相手キャラは既知だが、表記の分岐は明示しておく。
    let opponent_label = opponent_character.map(crate::detectors::display_character);
    let move_note = match repeated_move {
        Some((name, count)) => match &opponent_label {
            Some(label) => format!(
                "特に {label} の {name} は {count} 回ガードして、いずれも反撃していません。この技はガード後に反撃が確定します。"
            ),
            None => format!(
                "特に相手の {name} は {count} 回ガードして、いずれも反撃していません。この技はガード後に反撃が確定します。"
            ),
        },
        None => String::new(),
    };
    Some(AdviceCard {
        id: "punish_missed".to_string(),
        kind: AdviceKind::Diagnosis,
        confidence: EventConfidence::High,
        title: match missed.len() {
            1 => "確定反撃を見逃した場面",
            _ => "確定反撃を繰り返し見逃している",
        }.to_string(),
        severity: 0.04 * missed.len() as f32,
        // 損失は機会費用であり、この指摘が原因で失った HP ではない。
        hp_lost: None,
        description: format!(
            "相手の技をガードした後、フレーム上の反撃猶予があり、位置解析でも近距離だったのに反撃していない場面が {} 回あります。{}相手の危険な技を覚えて、ガードしたら反撃する意識を持ちましょう。{}",
            missed.len(), move_note, option_text
        ),
        practice: format!(
            "{}がよく振る技のうち、ガードして確反が取れるものを 2-3 個に絞って覚えましょう。トレモでその技をガード → 最速で確反、を反復して実戦でも無意識に確反をとれるようにしましょう。",
            match &opponent_label {
                Some(label) => format!("{label} "),
                None => "対戦相手".to_string(),
            },
        ),
        evidence: missed.iter().zip(&identified).map(|(punish, move_data)| EvidenceClip {
            frame: punish.frame,
            end_frame: None,
            label: match move_data {
                Some(move_data) => format!(
                    "R{} 相手の {} に確反見逃し +{}F",
                    punish.round_no, move_data.name, punish.advantage
                ),
                None => format!(
                    "R{} 確反見逃し +{}F / 近距離確認",
                    punish.round_no, punish.advantage
                ),
            },
        }).collect(),
    })
}

/// 距離を確認できなかった確反見逃し候補。ガード(接触で確認済み)後に
/// 実測の反撃猶予があり、攻撃していない場面が繰り返しあるときだけ、
/// 断定しない観察として提示する。先端ガードで届かなかった可能性が
/// 残るため、診断には昇格させない。
fn detect_unconfirmed_misses(
    events: &MatchEvents,
    own: u8,
    own_character: Option<&str>,
) -> Option<AdviceCard> {
    let unconfirmed: Vec<_> = events
        .punishes
        .iter()
        .filter(|punish| {
            punish.side == own
                && punish.outcome == PunishOutcome::Missed
                && punish.reachability == PunishReachability::Unknown
        })
        .collect();
    if unconfirmed.len() < crate::MIN_REPEATED_NEGATIVE_OUTCOMES {
        return None;
    }
    let min_advantage = unconfirmed
        .iter()
        .map(|punish| punish.advantage)
        .min()
        .unwrap_or(0);
    let option_text = super::options::failed_option_text(own_character, min_advantage);
    Some(AdviceCard {
        id: "punish_missed".to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::Medium,
        title: "確定反撃の猶予を見逃した可能性".to_string(),
        severity: 0.02 * unconfirmed.len() as f32,
        hp_lost: None,
        description: format!(
            "相手の技をガードした後、フレーム上は反撃の猶予があったのに攻撃していない場面が {} 回あります。距離までは確認できていないため、先端ガードで届かなかった可能性は残ります。各クリップで反撃が届く距離だったかを確認してください。{}",
            unconfirmed.len(),
            option_text
        ),
        practice: "クリップで距離を確認し、届いていた場面があれば「ガードしたら最速の確反」をトレモで反復します。先端ガードだった場面は、届く技への置き換えか、歩いてからの反撃を検討しましょう。".to_string(),
        evidence: unconfirmed
            .iter()
            .map(|punish| EvidenceClip {
                frame: punish.frame,
                end_frame: None,
                label: format!(
                    "R{} 確反猶予 +{}F を見逃した可能性(距離未確認)",
                    punish.round_no, punish.advantage
                ),
            })
            .collect(),
    })
}
