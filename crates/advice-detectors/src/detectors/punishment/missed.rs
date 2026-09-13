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
        return None;
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
    let move_note = match repeated_move {
        Some((name, count)) => format!(
            "特に相手の {name} は {count} 回ガードして、いずれも反撃していません。この技はガード後に反撃が確定します。"
        ),
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
        practice: "対戦相手がよく振る技のうち、ガードして確反が取れるものを 2-3 個に絞って覚えましょう。トレモでその技をガード → 最速で確反、を反復して実戦でも無意識に確反をとれるようにしましょう。".to_string(),
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
