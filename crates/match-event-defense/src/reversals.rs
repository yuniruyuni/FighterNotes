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
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();
    for s in 0..2usize {
        let me = s as u8 + 1;
        let own = &meter_state[s];
        let epochs = &meter_epoch[s];
        let mut i = 0usize;
        while i < n {
            if own[i] != MeterState::Invincible {
                i += 1;
                continue;
            }
            let start = i;
            let epoch = epochs.get(start).copied().unwrap_or(-1);
            let end = loop {
                while i < n
                    && own[i] == MeterState::Invincible
                    && epochs.get(i).copied() == Some(epoch)
                {
                    i += 1;
                }
                let run_end = i - 1;
                let merge_end = run_end
                    .saturating_add(REVERSAL_INV_MERGE_GAP)
                    .min(n.saturating_sub(1));
                let next_start = if i < n {
                    (i..=merge_end)
                        .take_while(|&frame| epochs.get(frame).copied() == Some(epoch))
                        .find(|&frame| own[frame] == MeterState::Invincible)
                } else {
                    None
                };
                let Some(next_start) = next_start else {
                    break run_end;
                };
                i = next_start;
            };
            if epoch < 0 || continuous_epoch(epochs, start, end) != Some(epoch) {
                continue;
            }
            let sf = features[start].frame_index;
            let ef = features[end].frame_index;

            // DIのアーマー表示と無敵技を混同しない。入力履歴でDIが確定して
            // いる場合はDriveImpactEventへ排他的に帰属させる。
            if segments[s].iter().any(|segment| {
                segment.is_drive_impact()
                    && segment.start_frame <= ef.saturating_add(10)
                    && segment.start_frame.saturating_add(45) >= sf
            }) {
                continue;
            }

            // Teleports also have an inv -> active signature. Once a
            // character-specific detector has identified the run, it must not
            // be reported as a failed DP/super.
            if teleports.iter().any(|teleport| {
                teleport.attacker == me
                    && teleport.inv_start_frame <= ef
                    && teleport.inv_end_frame >= sf
            }) {
                continue;
            }

            // 「技」の証拠: 無敵技（DP/SA）は空振りでも inv → 自分の Active が
            // 必ずメーターに出る。inv ラン直後に own Active が続かないランは
            // 投げ抜け・被投げ・起き上がり等のシステム無敵なので指摘しない
            // （実例: 2026-07-08 動画の THROW ESCAPE 演出 inv_full 42gf →
            // 直後被弾が「無敵技を狩られた」と誤検出されていた）。
            // 長さ上限は使わない: 本物の SA3 は演出込みで inv_full ≈100
            // video frames 続く（2026-06-16 f8532 実測）ため偽陰性になる
            let act_end = (end + 1 + REVERSAL_ACT_LOOKAHEAD).min(n);
            let is_move = own[end + 1..act_end]
                .iter()
                .enumerate()
                .any(|(offset, &state)| {
                    epochs.get(end + 1 + offset).copied() == Some(epoch)
                        && matches!(state, MeterState::Active | MeterState::ProjectileActive)
                });
            if !is_move {
                continue;
            }

            // ヒットしていれば無敵技は通っている → 指摘対象外
            let hit = contacts.iter().any(|c| {
                c.attacker == me
                    && c.hit
                    && c.frame >= sf
                    && c.frame <= ef + REVERSAL_WINDOW
                    && epochs.get(idx_of(features, c.frame)).copied() == Some(epoch)
            });
            if hit {
                continue;
            }
            let blocked = contacts.iter().any(|c| {
                c.attacker == me
                    && !c.hit
                    && c.frame >= sf
                    && c.frame <= ef + REVERSAL_WINDOW
                    && epochs.get(idx_of(features, c.frame)).copied() == Some(epoch)
            });
            // 後隙を狩られた（被弾した）ものだけを記録。無事に逃げ切った
            // 空振り無敵技はリスクが顕在化していないので指摘しない
            let drop = damage
                .iter()
                .filter(|d| {
                    d.victim == me
                        && d.start_frame >= ef
                        && d.start_frame <= ef + REVERSAL_PUNISH_WINDOW
                        && epochs.get(idx_of(features, d.start_frame)).copied() == Some(epoch)
                })
                .map(|d| d.drop)
                .fold(0.0f32, f32::max);
            if drop <= 0.0 {
                continue;
            }
            let explicit_dp = segments[s].iter().any(|segment| {
                segment.badges.iter().any(|badge| badge == "DP")
                    && segment.start_frame <= ef
                    && segment.start_frame.saturating_add(30) >= sf
            });
            let defensive_context = own[start.saturating_sub(45)..start]
                .contains(&MeterState::Stun)
                || contacts.iter().any(|contact| {
                    contact.victim == me
                        && contact.frame <= sf
                        && contact.frame.saturating_add(90) >= sf
                });
            if let Some(round_no) = round_of(rounds, sf) {
                out.push(ReversalEvent {
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
                });
            }
        }
    }
    out.sort_by_key(|r| r.frame);
    out
}
