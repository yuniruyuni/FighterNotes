//! ガード後の最速打撃／最速投げの抽出。
//!
//! 入力表示だけではなく、ガード硬直明け直後にメーター上の `Startup` が
//! 実際に始まったことを必須にする。メーターリセットをまたぐ候補、DI、
//! 無敵技は別イベントへ排他的に帰属させる。

use super::{
    continuous_epoch, round_of, AdvantageOutcome, AdvantageSituationEvent, ContactEvent,
    DamageEvent, DefensiveActionKind, EventConfidence, InputSegment, MeterState, MinusPressEvent,
    MinusPressOutcome, MinusSituationEvent, PressureFollowUp, RoundInfo, ADVANTAGE_ACTION_GRACE,
    ADVANTAGE_OUTCOME_WINDOW, ADVANTAGE_THRESHOLD, MINUS_PRESS_INV_WINDOW, MINUS_PRESS_MAX,
    MINUS_PRESS_OUTCOME_WINDOW, MINUS_PRESS_THRESHOLD,
};

const FASTEST_ACTION_GRACE: usize = 1;
const INPUT_LINK_WINDOW: u32 = 8;

fn same_epoch_at(epochs: &[Vec<i32>; 2], frame: usize, epoch: i32) -> bool {
    epochs[0].get(frame).copied() == Some(epoch) && epochs[1].get(frame).copied() == Some(epoch)
}

fn move_in_progress(state: &[MeterState], frame: usize) -> bool {
    let start = frame.saturating_sub(3);
    (start..=frame.min(state.len().saturating_sub(1))).any(|index| {
        matches!(
            state[index],
            MeterState::Startup | MeterState::Active | MeterState::Recovery
        )
    })
}

pub struct MinusEvents {
    pub(crate) presses: Vec<MinusPressEvent>,
    pub(crate) situations: Vec<MinusSituationEvent>,
    pub(crate) advantages: Vec<AdvantageSituationEvent>,
}

/// 入力欄がその瞬間に読めていたか。欠測を「何もしなかった」と誤認しない。
fn input_context_observed(segments: &[InputSegment], frame: u32) -> bool {
    segments.iter().any(|segment| {
        segment.evidence.has_direct_observation()
            && segment.start_frame <= frame.saturating_add(2)
            && segment.end_frame.saturating_add(2) >= frame
    })
}

struct AdvantageInputs<'a> {
    meter_epoch: &'a [Vec<i32>; 2],
    /// 有利側のメーター状態。
    state: &'a [MeterState],
    side: u8,
    index: usize,
    /// 有利側が行動可能になったフレーム。
    actionable: usize,
    /// ガードした側が行動可能になったフレーム。
    opponent_actionable: usize,
    plus_frames: u32,
    epoch: i32,
    contact: &'a ContactEvent,
    contacts: &'a [ContactEvent],
    damage: &'a [DamageEvent],
    segments: &'a [Vec<InputSegment>; 2],
    rounds: &'a [RoundInfo],
    used_frames: &'a mut Vec<u32>,
    frames: usize,
}

/// ガードさせて有利を取った側が、その有利のうちに攻めたかを判定する。
///
/// 攻撃を開始しなかった場合の結果は、続けて相手の攻撃を受ける側へ回ったか
/// （`TurnLost`）と、双方動かず仕切り直したか（`Reset`）だけを区別する。
/// 「前に歩いた」「様子を見た」の内訳は入力表示から断定できないため持たない。
fn extract_advantage(inputs: AdvantageInputs<'_>) -> Option<AdvantageSituationEvent> {
    let AdvantageInputs {
        meter_epoch,
        state,
        side,
        index,
        actionable,
        opponent_actionable,
        plus_frames,
        epoch,
        contact,
        contacts,
        damage,
        segments,
        rounds,
        used_frames,
        frames: n,
    } = inputs;
    if plus_frames < ADVANTAGE_THRESHOLD {
        return None;
    }
    let advantage_frame = actionable as u32;
    if used_frames.contains(&advantage_frame) {
        return None;
    }
    // 有利側の入力欄が読めていた機会だけを分母に入れる。
    if !input_context_observed(&segments[index], advantage_frame) {
        return None;
    }
    used_frames.push(advantage_frame);
    let round_no = round_of(rounds, advantage_frame).unwrap_or(0);

    // 相手が動けるようになるまでに発生が始まっていれば、有利を使っている。
    let action_end = (opponent_actionable + ADVANTAGE_ACTION_GRACE).min(n - 1);
    let action_start = (actionable..=action_end).find(|&frame| {
        same_epoch_at(meter_epoch, frame, epoch) && state[frame] == MeterState::Startup
    });

    if let Some(action_start) = action_start {
        let action_frame = action_start as u32;
        let input = segments[index]
            .iter()
            .filter(|segment| {
                segment.has_button()
                    && segment.evidence.has_direct_observation()
                    && segment.start_frame >= contact.frame
                    && segment.start_frame <= action_frame.saturating_add(2)
                    && segment.end_frame.saturating_add(INPUT_LINK_WINDOW) >= action_frame
            })
            .max_by_key(|segment| segment.start_frame);
        return Some(AdvantageSituationEvent {
            side,
            frame: advantage_frame,
            plus_frames,
            follow_up: input.map(|input| {
                if input.throw {
                    PressureFollowUp::Throw
                } else {
                    PressureFollowUp::Strike
                }
            }),
            action_frame: Some(action_frame),
            pressed: input.map_or_else(String::new, |input| input.badges.join("+")),
            outcome: AdvantageOutcome::Continued,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: contact.frame,
            round_no,
        });
    }

    // 攻めなかった場合、結果窓のうちに攻守が入れ替わったかだけを見る。
    let result_end = advantage_frame.saturating_add(ADVANTAGE_OUTCOME_WINDOW);
    let turn_lost = contacts.iter().any(|candidate| {
        candidate.attacker == contact.victim
            && candidate.victim == contact.attacker
            && candidate.frame > advantage_frame
            && candidate.frame <= result_end
            && same_epoch_at(meter_epoch, candidate.frame as usize, epoch)
    });
    let outcome = if turn_lost {
        AdvantageOutcome::TurnLost
    } else {
        AdvantageOutcome::Reset
    };
    let drop = if turn_lost {
        damage
            .iter()
            .filter(|event| {
                event.victim == contact.attacker
                    && event.start_frame > advantage_frame
                    && event.start_frame <= result_end
            })
            .map(|event| event.drop)
            .sum()
    } else {
        0.0
    };
    let window_end = (result_end as usize).min(n - 1);
    let confidence = if continuous_epoch(&meter_epoch[index], actionable, window_end) == Some(epoch)
        && continuous_epoch(&meter_epoch[1 - index], actionable, window_end) == Some(epoch)
    {
        EventConfidence::High
    } else {
        EventConfidence::Medium
    };
    Some(AdvantageSituationEvent {
        side,
        frame: advantage_frame,
        plus_frames,
        follow_up: None,
        action_frame: None,
        pressed: String::new(),
        outcome,
        drop,
        confidence,
        source_contact_frame: contact.frame,
        round_no,
    })
}

pub(crate) fn extract_minus_events(
    meter_state: &[Vec<MeterState>; 2],
    meter_epoch: &[Vec<i32>; 2],
    meter_game_frame: &[Vec<i64>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> MinusEvents {
    if meter_state[0].is_empty() || meter_epoch[0].is_empty() {
        return MinusEvents {
            presses: Vec::new(),
            situations: Vec::new(),
            advantages: Vec::new(),
        };
    }
    let n = meter_state[0].len();
    let mut presses = Vec::new();
    let mut situations = Vec::new();
    let mut advantages = Vec::new();
    let mut used_action_start: [Vec<u32>; 2] = [Vec::new(), Vec::new()];
    let mut used_situation_frame: [Vec<u32>; 2] = [Vec::new(), Vec::new()];
    let mut used_advantage_frame: [Vec<u32>; 2] = [Vec::new(), Vec::new()];

    for contact in contacts
        .iter()
        .filter(|contact| !contact.hit && !contact.projectile)
    {
        let victim_index = contact.victim as usize - 1;
        let opponent_index = 1 - victim_index;
        let own = &meter_state[victim_index];
        let opponent = &meter_state[opponent_index];
        let own_epoch = &meter_epoch[victim_index];
        let opponent_epoch = &meter_epoch[opponent_index];
        let start = contact.frame as usize;
        if start >= n {
            continue;
        }
        let epoch = own_epoch[start];
        if epoch < 0 || opponent_epoch.get(start).copied() != Some(epoch) {
            continue;
        }

        let mut stun_start = start;
        while stun_start < n
            && stun_start <= start + 3
            && same_epoch_at(meter_epoch, stun_start, epoch)
            && own[stun_start] != MeterState::Stun
        {
            stun_start += 1;
        }
        if stun_start >= n
            || own[stun_start] != MeterState::Stun
            || !same_epoch_at(meter_epoch, stun_start, epoch)
        {
            continue;
        }
        let mut own_actionable = stun_start;
        while own_actionable < n
            && same_epoch_at(meter_epoch, own_actionable, epoch)
            && own[own_actionable] == MeterState::Stun
        {
            own_actionable += 1;
        }
        if own_actionable >= n || !same_epoch_at(meter_epoch, own_actionable, epoch) {
            continue;
        }

        let mut opponent_actionable = start;
        while opponent_actionable < n
            && same_epoch_at(meter_epoch, opponent_actionable, epoch)
            && matches!(
                opponent[opponent_actionable],
                MeterState::Active
                    | MeterState::ProjectileActive
                    | MeterState::MotionRecovery
                    | MeterState::Recovery
            )
        {
            opponent_actionable += 1;
        }
        if opponent_actionable >= n
            || own_actionable <= opponent_actionable
            || !same_epoch_at(meter_epoch, opponent_actionable, epoch)
        {
            continue;
        }

        let minus = match (
            meter_game_frame[victim_index].get(own_actionable),
            meter_game_frame[opponent_index].get(opponent_actionable),
        ) {
            (Some(&own_frame), Some(&opponent_frame))
                if own_frame >= 0 && opponent_frame >= 0 && own_frame >= opponent_frame =>
            {
                (own_frame - opponent_frame) as u32
            }
            _ => (own_actionable - opponent_actionable) as u32,
        };
        if !(MINUS_PRESS_THRESHOLD..=MINUS_PRESS_MAX).contains(&minus) {
            continue;
        }

        // ── 有利側の攻め継続 ─────────────────────────────────────────────
        // 守備側の不利幅は、そのまま攻撃側の有利幅になる。守備側の判定に
        // 使う分岐へ入る前に処理し、片側の欠測でもう片側を落とさない。
        if let Some(advantage) = extract_advantage(AdvantageInputs {
            meter_epoch,
            state: opponent,
            side: contact.attacker,
            index: opponent_index,
            actionable: opponent_actionable,
            opponent_actionable: own_actionable,
            plus_frames: minus,
            epoch,
            contact,
            contacts,
            damage,
            segments,
            rounds,
            used_frames: &mut used_advantage_frame[opponent_index],
            frames: n,
        }) {
            advantages.push(advantage);
        }

        let situation_frame = own_actionable as u32;
        if used_situation_frame[victim_index].contains(&situation_frame) {
            continue;
        }
        // 「何もしなかった」を分母に入れるには、その瞬間の入力欄が読めて
        // いたことが必要。欠測をガード継続と誤認しない。
        if !input_context_observed(&segments[victim_index], situation_frame) {
            continue;
        }
        used_situation_frame[victim_index].push(situation_frame);
        let round_no = round_of(rounds, situation_frame).unwrap_or(0);
        let no_fast_action = || MinusSituationEvent {
            side: contact.victim,
            frame: situation_frame,
            minus_frames: minus,
            fastest_action: None,
            action_frame: None,
            pressed: String::new(),
            outcome: None,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: contact.frame,
            round_no,
        };

        let action_end = (own_actionable + FASTEST_ACTION_GRACE).min(n - 1);
        let Some(action_start) = (own_actionable..=action_end).find(|&frame| {
            same_epoch_at(meter_epoch, frame, epoch) && own[frame] == MeterState::Startup
        }) else {
            situations.push(no_fast_action());
            continue;
        };
        let action_frame = action_start as u32;
        if used_action_start[victim_index].contains(&action_frame) {
            situations.push(no_fast_action());
            continue;
        }

        let input = segments[victim_index]
            .iter()
            .filter(|segment| {
                segment.has_button()
                    && segment.evidence.has_direct_observation()
                    && !segment.is_drive_impact()
                    && !segment.badges.iter().any(|badge| badge == "DP")
                    && segment.start_frame >= contact.frame
                    && segment.start_frame <= action_frame.saturating_add(2)
                    && segment.end_frame.saturating_add(INPUT_LINK_WINDOW) >= action_frame
            })
            .max_by_key(|segment| segment.start_frame);
        let Some(input) = input else {
            // DI・DP・入力欠測など、通常の最速打撃／投げへ帰属できない
            // 行動は分母には残すが、特定の回答としては数えない。
            situations.push(no_fast_action());
            continue;
        };
        let invincible =
            (action_start..(action_start + MINUS_PRESS_INV_WINDOW).min(n)).any(|frame| {
                same_epoch_at(meter_epoch, frame, epoch) && own[frame] == MeterState::Invincible
            });
        if invincible || opponent[action_start] == MeterState::Recovery {
            situations.push(no_fast_action());
            continue;
        }

        let result_end = action_frame.saturating_add(MINUS_PRESS_OUTCOME_WINDOW);
        let lost_contact = contacts.iter().find(|candidate| {
            candidate.attacker == contact.attacker
                && candidate.victim == contact.victim
                && candidate.hit
                && candidate.frame >= action_frame
                && candidate.frame <= result_end
                && same_epoch_at(meter_epoch, candidate.frame as usize, epoch)
                && move_in_progress(own, candidate.frame as usize)
        });
        let won_contact = contacts.iter().find(|candidate| {
            candidate.attacker == contact.victim
                && candidate.frame >= action_frame
                && candidate.frame <= result_end
                && same_epoch_at(meter_epoch, candidate.frame as usize, epoch)
        });
        let fallback_damage = damage.iter().find(|event| {
            event.victim == contact.victim
                && event.start_frame >= action_frame
                && event.start_frame <= result_end
                && move_in_progress(own, event.start_frame as usize)
        });

        let (outcome, drop, confidence) = if let Some(hit) = lost_contact {
            let drop = damage
                .iter()
                .find(|event| {
                    event.victim == contact.victim
                        && event.start_frame + 5 >= hit.frame
                        && event.start_frame <= hit.frame + 25
                })
                .map_or(0.0, |event| event.drop);
            (MinusPressOutcome::CounterHit, drop, EventConfidence::High)
        } else if won_contact.is_some() {
            (MinusPressOutcome::Won, 0.0, EventConfidence::High)
        } else if let Some(event) = fallback_damage {
            (
                MinusPressOutcome::CounterHit,
                event.drop,
                EventConfidence::Medium,
            )
        } else {
            let end = (result_end as usize).min(n - 1);
            let confidence = if continuous_epoch(own_epoch, action_start, end) == Some(epoch)
                && continuous_epoch(opponent_epoch, action_start, end) == Some(epoch)
            {
                EventConfidence::High
            } else {
                EventConfidence::Medium
            };
            (MinusPressOutcome::GotAway, 0.0, confidence)
        };
        let pressed = if input.throw {
            "投げ".to_string()
        } else if input.badges.is_empty() {
            if input.auto {
                "AUTO".to_string()
            } else {
                "?".to_string()
            }
        } else {
            input.badges.join("+")
        };
        let action_kind = if input.throw {
            DefensiveActionKind::Throw
        } else {
            DefensiveActionKind::Strike
        };
        used_action_start[victim_index].push(action_frame);
        presses.push(MinusPressEvent {
            side: contact.victim,
            frame: action_frame,
            minus_frames: minus,
            pressed: pressed.clone(),
            action_kind,
            outcome,
            drop,
            confidence,
            source_contact_frame: contact.frame,
            round_no,
        });
        situations.push(MinusSituationEvent {
            side: contact.victim,
            frame: situation_frame,
            minus_frames: minus,
            fastest_action: Some(action_kind),
            action_frame: Some(action_frame),
            pressed,
            outcome: Some(outcome),
            drop,
            confidence,
            source_contact_frame: contact.frame,
            round_no,
        });
    }
    presses.sort_by_key(|event| event.frame);
    situations.sort_by_key(|event| event.frame);
    advantages.sort_by_key(|event| event.frame);
    MinusEvents {
        presses,
        situations,
        advantages,
    }
}

/// 既存の単体テストと内部呼び出し向け互換ラッパー。
#[cfg(any(test, feature = "test-support"))]
pub fn extract_presses_while_minus(
    meter_state: &[Vec<MeterState>; 2],
    meter_epoch: &[Vec<i32>; 2],
    meter_game_frame: &[Vec<i64>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> Vec<MinusPressEvent> {
    extract_minus_events(
        meter_state,
        meter_epoch,
        meter_game_frame,
        contacts,
        damage,
        segments,
        rounds,
    )
    .presses
}
