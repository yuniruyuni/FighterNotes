//! 入力をメーター上の実行状態と結果へ結び付ける共通行動イベント。

use super::*;

const ACTION_INPUT_LOOKBACK: usize = 3;
const ACTION_START_WINDOW: usize = 20;
const ACTION_ACTIVE_WINDOW: usize = 28;
/// DI は 26 ゲームフレーム発生で、ヒットストップ中は同じゲームフレームが
/// 複数の動画フレームへ伸びる。通常行動の窓とは分けて Active まで追う。
const DI_ACTIVE_WINDOW: usize = 80;
const THROW_DAMAGE_WINDOW: u32 = 125;
const THROW_DEDUP_WINDOW: u32 = 18;
const THROW_CONTACT_WINDOW: usize = 10;
const THROW_TECH_WINDOW: usize = 80;
const THROW_MAX_STARTUP_TO_ACTIVE: usize = 12;
const THROW_INVINCIBLE_LOOKBACK: usize = 3;
const THROW_INTERRUPT_DAMAGE_WINDOW: u32 = 90;
const DI_CONTACT_BACK: u32 = 4;
const DI_CONTACT_FWD: u32 = 14;
const DI_RESULT_WINDOW: u32 = 80;
const DI_DEDUP_WINDOW: u32 = 12;
const DRIVE_RUSH_INPUT_BACK: u32 = 8;
const DRIVE_RUSH_INPUT_FWD: u32 = 18;
const DRIVE_RUSH_RESULT_WINDOW: u32 = 90;
const DRIVE_RUSH_MIN_DRIVE_DROP: f32 = 0.05;

fn action_run(
    state: &[MeterState],
    epochs: &[i32],
    input_frame: u32,
    start_states: &[MeterState],
    active_window: usize,
) -> (Option<usize>, Option<usize>, Option<i32>) {
    let n = state.len().min(epochs.len());
    if n == 0 {
        return (None, None, None);
    }
    let input = (input_frame as usize).min(n - 1);
    let lo = input.saturating_sub(ACTION_INPUT_LOOKBACK);
    let hi = (input + ACTION_START_WINDOW).min(n - 1);
    let start = (lo..=hi).find(|&frame| start_states.contains(&state[frame]));
    let Some(start) = start else {
        return (None, None, None);
    };
    let epoch = epochs[start];
    if epoch < 0 {
        return (Some(start), None, None);
    }
    let active_end = (start + active_window).min(n - 1);
    let active = (start..=active_end).find(|&frame| {
        epochs[frame] == epoch
            && matches!(
                state[frame],
                MeterState::Active | MeterState::ProjectileActive
            )
    });
    (Some(start), active, Some(epoch))
}

pub(crate) fn extract_throw_actions(
    meter_state: &[Vec<MeterState>; 2],
    meter_epoch: &[Vec<i32>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> Vec<ThrowActionEvent> {
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();
    for side_index in 0..2usize {
        let thrower = side_index as u8 + 1;
        let victim = 3 - thrower;
        let mut consumed_until = 0u32;
        for segment in segments[side_index].iter().filter(|segment| segment.throw) {
            if segment.start_frame <= consumed_until {
                continue;
            }
            let Some(round_no) = round_of(rounds, segment.start_frame) else {
                continue;
            };
            let (startup, active, epoch) = action_run(
                &meter_state[side_index],
                &meter_epoch[side_index],
                segment.start_frame,
                &[MeterState::Startup],
                ACTION_ACTIVE_WINDOW,
            );
            let startup_frame = startup.map(|frame| frame as u32);
            let active_frame = active.map(|frame| frame as u32);
            let (outcome, damage_amount, confidence) = match (active, epoch) {
                (Some(active), Some(epoch)) => {
                    let active_u32 = active as u32;
                    let throw_animation_confirmed = startup.is_some_and(|startup| {
                        active.saturating_sub(startup) <= THROW_MAX_STARTUP_TO_ACTIVE
                    });
                    let contact_end = (active + THROW_CONTACT_WINDOW).min(n - 1);
                    let opponent_stunned = (active..=contact_end).any(|frame| {
                        meter_epoch[1 - side_index][frame] == epoch
                            && meter_state[1 - side_index][frame] == MeterState::Stun
                    });
                    let contact_seen = contacts.iter().any(|contact| {
                        contact.attacker == thrower
                            && contact.frame >= active_u32.saturating_sub(2)
                            && contact.frame <= contact_end as u32
                    });
                    let damage_amount: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == victim
                                && event.round_no == round_no
                                && event.start_frame >= active_u32
                                && event.start_frame <= active_u32 + THROW_DAMAGE_WINDOW
                        })
                        .map(|event| event.drop)
                        .sum();
                    let tech_end = (active + THROW_TECH_WINDOW).min(n - 1);
                    let mutual_invincibility = (active..=tech_end).any(|frame| {
                        meter_epoch[side_index][frame] == epoch
                            && meter_epoch[1 - side_index][frame] == epoch
                            && meter_state[side_index][frame] == MeterState::Invincible
                            && meter_state[1 - side_index][frame] == MeterState::Invincible
                    });
                    let invincible_start = active.saturating_sub(THROW_INVINCIBLE_LOOKBACK);
                    let opponent_invincible = (invincible_start..=contact_end).any(|frame| {
                        meter_epoch[1 - side_index][frame] == epoch
                            && meter_state[1 - side_index][frame] == MeterState::Invincible
                    });
                    let damage_to_thrower: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == thrower
                                && event.round_no == round_no
                                && event.start_frame >= active_u32.saturating_sub(2)
                                && event.start_frame <= active_u32 + THROW_INTERRUPT_DAMAGE_WINDOW
                        })
                        .map(|event| event.drop)
                        .sum();
                    if damage_amount >= THROW_MIN_DROP && (opponent_stunned || contact_seen) {
                        (ThrowOutcome::Hit, damage_amount, EventConfidence::High)
                    } else if throw_animation_confirmed
                        && opponent_invincible
                        && damage_to_thrower >= THROW_MIN_DROP
                    {
                        (
                            ThrowOutcome::InterruptedByInvincible,
                            0.0,
                            EventConfidence::High,
                        )
                    } else if mutual_invincibility && damage_amount < THROW_MIN_DROP {
                        (ThrowOutcome::Teched, 0.0, EventConfidence::High)
                    } else if throw_animation_confirmed && !opponent_stunned && !contact_seen {
                        (ThrowOutcome::ExecutedWhiff, 0.0, EventConfidence::High)
                    } else {
                        (
                            ThrowOutcome::Unconfirmed,
                            damage_amount,
                            EventConfidence::Medium,
                        )
                    }
                }
                _ => (ThrowOutcome::Unconfirmed, 0.0, EventConfidence::Low),
            };
            consumed_until = active_frame
                .unwrap_or(segment.start_frame)
                .saturating_add(THROW_DEDUP_WINDOW);
            out.push(ThrowActionEvent {
                thrower,
                input_frame: segment.start_frame,
                startup_frame,
                active_frame,
                outcome,
                damage: damage_amount,
                approach: ThrowApproach::Unknown,
                confidence,
                round_no,
            });
        }
    }
    out.sort_by_key(|event| event.input_frame);
    out
}

pub(crate) fn extract_drive_impacts(
    meter_state: &[Vec<MeterState>; 2],
    meter_epoch: &[Vec<i32>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> Vec<DriveImpactEvent> {
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();
    for side_index in 0..2usize {
        let side = side_index as u8 + 1;
        let opponent = 3 - side;
        let mut consumed_until = 0u32;
        for segment in segments[side_index]
            .iter()
            .filter(|segment| segment.is_drive_impact())
        {
            if segment.start_frame <= consumed_until {
                continue;
            }
            let Some(round_no) = round_of(rounds, segment.start_frame) else {
                continue;
            };
            // DI のアーマー区間はメーター上で Startup ではなく
            // Invincible として現れ、そのまま Active へ遷移する。
            let (_, active, epoch) = action_run(
                &meter_state[side_index],
                &meter_epoch[side_index],
                segment.start_frame,
                &[
                    MeterState::Startup,
                    MeterState::Invincible,
                    MeterState::Parry,
                ],
                DI_ACTIVE_WINDOW,
            );
            let active_frame = active.map(|frame| frame as u32);
            let (contact_frame, outcome, damage_amount, confidence) = match (active, epoch) {
                (Some(active), Some(epoch)) => {
                    let active_u32 = active as u32;
                    let contact = contacts
                        .iter()
                        .filter(|contact| {
                            contact.attacker == side
                                && contact.frame >= active_u32.saturating_sub(DI_CONTACT_BACK)
                                && contact.frame <= active_u32 + DI_CONTACT_FWD
                        })
                        .min_by_key(|contact| contact.frame);
                    let opponent_parry = {
                        let lo = active.saturating_sub(DI_CONTACT_BACK as usize);
                        let hi = (active + DI_CONTACT_FWD as usize).min(n - 1);
                        (lo..=hi).any(|frame| {
                            meter_epoch[1 - side_index][frame] == epoch
                                && meter_state[1 - side_index][frame] == MeterState::Parry
                        })
                    };
                    let damage_to_opponent: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == opponent
                                && event.round_no == round_no
                                && event.start_frame >= active_u32.saturating_sub(2)
                                && event.start_frame <= active_u32 + DI_RESULT_WINDOW
                        })
                        .map(|event| event.drop)
                        .sum();
                    let damage_to_self: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == side
                                && event.round_no == round_no
                                && event.start_frame >= active_u32
                                && event.start_frame <= active_u32 + DI_RESULT_WINDOW
                        })
                        .map(|event| event.drop)
                        .sum();
                    let counter_di = segments[1 - side_index].iter().any(|other| {
                        other.is_drive_impact()
                            && other.start_frame + 4 >= segment.start_frame
                            && other.start_frame <= segment.start_frame + 35
                    });
                    if opponent_parry && damage_to_opponent < DMG_MIN_DROP {
                        (
                            None,
                            DriveImpactOutcome::Parried,
                            0.0,
                            EventConfidence::High,
                        )
                    } else if counter_di && damage_to_self >= DMG_MIN_DROP {
                        (
                            None,
                            DriveImpactOutcome::Countered,
                            damage_to_self,
                            EventConfidence::High,
                        )
                    } else if let Some(contact) = contact {
                        if contact.hit || damage_to_opponent >= DMG_MIN_DROP {
                            (
                                Some(contact.frame),
                                DriveImpactOutcome::Hit,
                                damage_to_opponent,
                                EventConfidence::High,
                            )
                        } else {
                            (
                                Some(contact.frame),
                                DriveImpactOutcome::Blocked,
                                0.0,
                                EventConfidence::High,
                            )
                        }
                    } else {
                        (
                            None,
                            DriveImpactOutcome::Whiffed,
                            0.0,
                            EventConfidence::Medium,
                        )
                    }
                }
                _ => (
                    None,
                    DriveImpactOutcome::Unconfirmed,
                    0.0,
                    EventConfidence::Low,
                ),
            };
            consumed_until = active_frame
                .unwrap_or(segment.start_frame)
                .saturating_add(DI_DEDUP_WINDOW);
            out.push(DriveImpactEvent {
                side,
                input_frame: segment.start_frame,
                active_frame,
                contact_frame,
                outcome,
                damage: damage_amount,
                confidence,
                round_no,
            });
        }
    }
    out.sort_by_key(|event| event.input_frame);
    out
}

pub(crate) fn extract_drive_rushes(
    features: &[FrameFeatures],
    meter_state: &[Vec<MeterState>; 2],
    meter_epoch: &[Vec<i32>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> Vec<DriveRushEvent> {
    if features.is_empty() || meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len().min(features.len());
    let drive_ratio = |side_index: usize, frame: usize| -> f32 {
        let feature = &features[frame.min(features.len() - 1)];
        if side_index == 0 {
            feature.left_drive_ratio
        } else {
            feature.right_drive_ratio
        }
    };
    let mut out = Vec::new();
    for side_index in 0..2usize {
        let side = side_index as u8 + 1;
        let mut frame = 0usize;
        while frame < n {
            if meter_state[side_index][frame] != MeterState::Parry {
                frame += 1;
                continue;
            }
            let start = frame;
            let epoch = meter_epoch[side_index].get(start).copied().unwrap_or(-1);
            while frame < n
                && meter_state[side_index][frame] == MeterState::Parry
                && meter_epoch[side_index].get(frame).copied() == Some(epoch)
            {
                frame += 1;
            }
            let end = frame.saturating_sub(1);
            if epoch < 0 {
                continue;
            }
            let start_frame = features[start].frame_index;
            let end_frame = features[end].frame_index;
            let input = segments[side_index]
                .iter()
                .filter(|segment| matches!(segment.dir.as_str(), "L" | "R"))
                .filter(|segment| {
                    segment.start_frame.saturating_add(DRIVE_RUSH_INPUT_BACK) >= start_frame
                        && segment.start_frame <= end_frame.saturating_add(DRIVE_RUSH_INPUT_FWD)
                })
                .min_by_key(|segment| segment.start_frame.abs_diff(start_frame));
            let Some(input) = input else {
                continue;
            };
            let before_index = start.saturating_sub(5);
            let after_index = (end + 45).min(n - 1);
            let before = drive_ratio(side_index, before_index);
            let after_min = (end..=after_index)
                .map(|index| drive_ratio(side_index, index))
                .fold(before, f32::min);
            if before - after_min < DRIVE_RUSH_MIN_DRIVE_DROP {
                continue;
            }
            let Some(round_no) = round_of(rounds, input.start_frame) else {
                continue;
            };
            let raw = !contacts.iter().any(|contact| {
                contact.attacker == side
                    && contact.frame < start_frame
                    && contact.frame.saturating_add(30) >= start_frame
            });
            let result = contacts
                .iter()
                .filter(|contact| {
                    contact.frame >= input.start_frame
                        && contact.frame <= input.start_frame + DRIVE_RUSH_RESULT_WINDOW
                        && (contact.attacker == side || contact.victim == side)
                })
                .min_by_key(|contact| contact.frame);
            let (outcome, contact_frame, damage_amount) = if let Some(contact) = result {
                if contact.attacker == side {
                    let amount: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == 3 - side
                                && event.start_frame + 5 >= contact.frame
                                && event.start_frame <= contact.frame + 25
                        })
                        .map(|event| event.drop)
                        .sum();
                    (
                        if contact.hit {
                            DriveRushOutcome::Hit
                        } else {
                            DriveRushOutcome::Blocked
                        },
                        Some(contact.frame),
                        amount,
                    )
                } else {
                    let amount: f32 = damage
                        .iter()
                        .filter(|event| {
                            event.victim == side
                                && event.start_frame + 5 >= contact.frame
                                && event.start_frame <= contact.frame + 25
                        })
                        .map(|event| event.drop)
                        .sum();
                    (DriveRushOutcome::Stopped, Some(contact.frame), amount)
                }
            } else {
                (DriveRushOutcome::NoContact, None, 0.0)
            };
            out.push(DriveRushEvent {
                side,
                frame: input.start_frame,
                raw,
                outcome,
                contact_frame,
                damage: damage_amount,
                // 前後方向は空間二段目で確定する。
                confidence: EventConfidence::Medium,
                round_no,
            });
        }
    }
    out.sort_by_key(|event| event.frame);
    out
}
