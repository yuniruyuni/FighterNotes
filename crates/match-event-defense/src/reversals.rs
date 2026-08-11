//! 無敵技（DP/SA）を狩られたイベントの抽出
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;

pub struct ReversalInputs<'a> {
    pub features: &'a [FrameFeatures],
    pub meter_state: &'a [Vec<MeterState>; 2],
    pub meter_epoch: &'a [Vec<i32>; 2],
    pub contacts: &'a [ContactEvent],
    pub damage: &'a [DamageEvent],
    pub segments: &'a [Vec<InputSegment>; 2],
    pub rounds: &'a [RoundInfo],
    pub teleports: &'a [TeleportEvent],
}

/// 無敵技ぶっぱ被弾を抽出する。
///
/// 自分の Invincible run（DP/SA リバーサルの発生・無敵）を列挙し、
/// ヒットせず（ガード or 空振り）に後隙を狩られた（被弾した）ものを記録する。
pub fn extract_reversals(inputs: ReversalInputs<'_>) -> Vec<ReversalEvent> {
    let ReversalInputs {
        features,
        meter_state,
        meter_epoch,
        contacts,
        damage,
        segments,
        rounds,
        teleports,
    } = inputs;
    let mut out = Vec::new();
    for s in 0..2usize {
        let me = s as u8 + 1;
        let own = &meter_state[s];
        let epochs = &meter_epoch[s];
        for (epoch, start, end) in invincible_runs(own, epochs) {
            #[allow(clippy::too_many_arguments)]
            fn event_from_run(
                epoch: i32,
                start: usize,
                end: usize,
                me: u8,
                own: &[MeterState],
                epochs: &[i32],
                features: &[FrameFeatures],
                own_segments: &[InputSegment],
                contacts: &[ContactEvent],
                damage: &[DamageEvent],
                teleports: &[TeleportEvent],
                rounds: &[RoundInfo],
            ) -> Option<ReversalEvent> {
                if epoch < 0 {
                    return None;
                }
                if continuous_epoch(epochs, start, end) != Some(epoch) {
                    return None;
                }
                let sf = features[start].frame_index;
                let ef = features[end].frame_index;

                // DIのアーマー表示と無敵技を混同しない。入力履歴でDIが確定して
                // いる場合はDriveImpactEventへ排他的に帰属させる。
                if own_segments.iter().any(|segment| {
                    segment.is_drive_impact()
                        && segment.start_frame <= ef.saturating_add(10)
                        && segment.start_frame.saturating_add(45) >= sf
                }) {
                    return None;
                }

                // Teleports also have an inv -> active signature. Once a
                // character-specific detector has identified the run, it must not
                // be reported as a failed DP/super.
                if teleports.iter().any(|teleport| {
                    teleport.attacker == me
                        && teleport.inv_start_frame <= ef
                        && teleport.inv_end_frame >= sf
                }) {
                    return None;
                }

                // 「技」の証拠: 無敵技（DP/SA）は空振りでも inv → 自分の Active が
                // 必ずメーターに出る。inv ラン直後に own Active が続かないランは
                // 投げ抜け・被投げ・起き上がり等のシステム無敵なので指摘しない
                // （実例: 2026-07-08 動画の THROW ESCAPE 演出 inv_full 42gf →
                // 直後被弾が「無敵技を狩られた」と誤検出されていた）。
                // 長さ上限は使わない: 本物の SA3 は演出込みで inv_full ≈100
                // video frames 続く（2026-06-16 f8532 実測）ため偽陰性になる
                if !has_followup_attack(own, epochs, epoch, end) {
                    return None;
                }

                // ヒットしていれば無敵技は通っている → 指摘対象外
                let hit = contacts.iter().any(|contact| {
                    contact.attacker == me
                        && contact.hit
                        && contact.frame >= sf
                        && contact.frame <= ef + REVERSAL_WINDOW
                        && epochs.get(idx_of(features, contact.frame)).copied() == Some(epoch)
                });
                if hit {
                    return None;
                }
                let blocked = contacts.iter().any(|contact| {
                    contact.attacker == me
                        && !contact.hit
                        && contact.frame >= sf
                        && contact.frame <= ef + REVERSAL_WINDOW
                        && epochs.get(idx_of(features, contact.frame)).copied() == Some(epoch)
                });
                // 後隙を狩られた（被弾した）ものだけを記録。無事に逃げ切った
                // 空振り無敵技はリスクが顕在化していないので指摘しない
                let drop = damage
                    .iter()
                    .filter(|event| {
                        event.victim == me
                            && event.start_frame >= ef
                            && event.start_frame <= ef + REVERSAL_PUNISH_WINDOW
                            && epochs.get(idx_of(features, event.start_frame)).copied()
                                == Some(epoch)
                    })
                    .map(|event| event.drop)
                    .fold(0.0f32, f32::max);
                if drop <= 0.0 {
                    return None;
                }
                let explicit_dp = own_segments.iter().any(|segment| {
                    segment.badges.iter().any(|badge| badge == "DP")
                        && segment.start_frame <= ef
                        && segment.start_frame.saturating_add(30) >= sf
                });
                let defensive_context = had_recent_stun(own, start)
                    || contacts.iter().any(|contact| {
                        contact.victim == me
                            && contact.frame <= sf
                            && contact.frame.saturating_add(90) >= sf
                    });
                let round_no = round_of(rounds, sf)?;
                Some(ReversalEvent {
                    side: me,
                    frame: sf,
                    drop,
                    blocked,
                    confidence: if explicit_dp || defensive_context {
                        EventConfidence::High
                    } else {
                        EventConfidence::Medium
                    },
                    round_no,
                })
            }
            let event = event_from_run(
                epoch,
                start,
                end,
                me,
                own,
                epochs,
                features,
                &segments[s],
                contacts,
                damage,
                teleports,
                rounds,
            );
            if let Some(event) = event {
                out.push(event);
            }
        }
    }
    out.sort_by_key(|r| r.frame);
    out
}

/// Collect invincibility runs in one bounded pass. Short interruptions are
/// merged only while the meter epoch stays continuous.
fn invincible_runs(own: &[MeterState], epochs: &[i32]) -> Vec<(i32, usize, usize)> {
    let mut runs = Vec::new();
    let mut pending: Option<(i32, usize, usize)> = None;

    for (frame, &state) in own.iter().enumerate() {
        let epoch = epochs.get(frame).copied();
        let should_close = pending.is_some_and(|(pending_epoch, _, end)| {
            epoch != Some(pending_epoch) || frame > end.saturating_add(REVERSAL_INV_MERGE_GAP)
        });
        if should_close {
            if let Some(run) = pending.take() {
                runs.push(run);
            }
        }

        if state == MeterState::Invincible {
            if let Some(epoch) = epoch {
                if let Some((_, _, end)) = pending.as_mut() {
                    *end = frame;
                } else {
                    pending = Some((epoch, frame, frame));
                }
            }
        }
    }
    if let Some(run) = pending {
        runs.push(run);
    }
    runs
}

fn has_followup_attack(own: &[MeterState], epochs: &[i32], epoch: i32, end: usize) -> bool {
    own.iter()
        .zip(epochs)
        .skip(end)
        .skip(1)
        .take(REVERSAL_ACT_LOOKAHEAD)
        .any(|(&state, &state_epoch)| {
            state_epoch == epoch
                && matches!(state, MeterState::Active | MeterState::ProjectileActive)
        })
}

fn had_recent_stun(own: &[MeterState], start: usize) -> bool {
    own.iter()
        .take(start)
        .rev()
        .take(45)
        .any(|state| *state == MeterState::Stun)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::test_support::feat;

    fn extract(own: Vec<MeterState>) -> Vec<ReversalEvent> {
        let length = own.len();
        let features: Vec<_> = (0..length)
            .map(|frame| feat(frame as u32, 1.0, 1.0))
            .collect();
        extract_reversals(ReversalInputs {
            features: &features,
            meter_state: &[own, vec![MeterState::Free; length]],
            meter_epoch: &[vec![0; length], vec![0; length]],
            contacts: &[],
            damage: &[],
            segments: &[vec![], vec![]],
            rounds: &[],
            teleports: &[],
        })
    }

    #[test]
    fn scanner_handles_an_empty_series_and_invincibility_at_the_last_frame() {
        assert!(extract(vec![]).is_empty());
        assert!(extract(vec![MeterState::Invincible]).is_empty());
    }

    #[test]
    fn invincibility_runs_merge_only_inside_the_same_epoch_and_gap() {
        let mut exact_gap = vec![MeterState::Free; REVERSAL_INV_MERGE_GAP + 1];
        exact_gap[0] = MeterState::Invincible;
        exact_gap[REVERSAL_INV_MERGE_GAP] = MeterState::Invincible;
        assert_eq!(
            invincible_runs(&exact_gap, &vec![0; exact_gap.len()]),
            vec![(0, 0, REVERSAL_INV_MERGE_GAP)]
        );

        let mut beyond_gap = vec![MeterState::Free; REVERSAL_INV_MERGE_GAP + 2];
        beyond_gap[0] = MeterState::Invincible;
        beyond_gap[REVERSAL_INV_MERGE_GAP + 1] = MeterState::Invincible;
        assert_eq!(
            invincible_runs(&beyond_gap, &vec![0; beyond_gap.len()]),
            vec![
                (0, 0, 0),
                (0, REVERSAL_INV_MERGE_GAP + 1, REVERSAL_INV_MERGE_GAP + 1)
            ]
        );

        assert_eq!(
            invincible_runs(&[MeterState::Invincible, MeterState::Invincible], &[0, 1]),
            vec![(0, 0, 0), (1, 1, 1)]
        );
    }
}
