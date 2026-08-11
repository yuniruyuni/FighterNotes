//! ダウンと、その起き上がりに対する攻めの抽出。
//!
//! フレームメーターにダウン専用の表示は無く、やられもガード硬直もダウンも
//! 同じ `Stun` になる。長さだけで分けると、長いガード連係や長いコンボが
//! ダウンとして混ざる。
//!
//! ここでは「攻撃側は動けるようになったのに、相手はまだ `Stun` のまま」
//! という空白が続くことを必須にする。連続ガードや連続ヒットの最中は攻撃側も
//! 拘束されるため、この空白はダウン特有であり、同時に起き攻めの準備時間
//! そのものになる。

use super::runs::{runs_of, MeterRun};
use super::{
    round_of, ContactEvent, EventConfidence, KnockdownEvent, MeterState, OkizemeOutcome, RoundInfo,
    KNOCKDOWN_CAUSE_GRACE, KNOCKDOWN_MIN_SETUP, KNOCKDOWN_MIN_STUN, OKIZEME_PRESSURE_WINDOW,
};
use crate::frame_features::FrameFeatures;

pub struct KnockdownInputs<'a> {
    pub features: &'a [FrameFeatures],
    pub meter_state: &'a [Vec<MeterState>; 2],
    pub meter_epoch: &'a [Vec<i32>; 2],
    pub contacts: &'a [ContactEvent],
    pub rounds: &'a [RoundInfo],
}

pub fn extract_knockdowns(inputs: KnockdownInputs<'_>) -> Vec<KnockdownEvent> {
    let KnockdownInputs {
        features,
        meter_state,
        meter_epoch,
        contacts,
        rounds,
    } = inputs;
    let n = meter_state[0].len();
    let Some(last_index) = n.checked_sub(1) else {
        return Vec::new();
    };
    let mut out = Vec::new();

    for down_index in 0..2usize {
        let down_side = down_index as u8 + 1;
        let attacker_side = 3 - down_side;
        let down = &meter_state[down_index];
        let attacker = &meter_state[1 - down_index];
        let down_epoch = &meter_epoch[down_index];
        let attacker_epoch = &meter_epoch[1 - down_index];

        for MeterRun { start, end, epoch } in runs_of(down, down_epoch, MeterState::Stun) {
            #[allow(clippy::too_many_arguments)]
            fn event_from_run(
                start: usize,
                end: usize,
                epoch: i32,
                last_index: usize,
                frames: usize,
                down_side: u8,
                attacker_side: u8,
                attacker: &[MeterState],
                attacker_epoch: &[i32],
                features: &[FrameFeatures],
                contacts: &[ContactEvent],
                rounds: &[RoundInfo],
            ) -> Option<KnockdownEvent> {
                if end + 1 - start < KNOCKDOWN_MIN_STUN {
                    return None;
                }

                // 攻撃側が自由に動けるのに相手はまだ倒れている区間。
                // 連続ガード・連続ヒット中は攻撃側も拘束されるため成立しない。
                let setup_frames = (start..=end)
                    .filter(|&frame| {
                        attacker_epoch.get(frame).copied() == Some(epoch)
                            && matches!(attacker[frame], MeterState::Free)
                    })
                    .count();
                if setup_frames < KNOCKDOWN_MIN_SETUP {
                    return None;
                }

                let start_frame = features[start].frame_index;
                // 起き上がりは stun 終端の次のフレーム。系列末尾なら結果を見ない。
                if end >= last_index {
                    return None;
                }
                let wakeup = end + 1;
                let wakeup_frame = features[wakeup].frame_index;
                let round_no = round_of(rounds, start_frame)?;

                // ダウンさせたのが誰かは、原因のヒット接触から取る。
                let caused = contacts.iter().any(|contact| {
                    contact.hit
                        && contact.victim == down_side
                        && contact.attacker == attacker_side
                        && contact.frame + KNOCKDOWN_CAUSE_GRACE >= start_frame
                        && contact.frame <= features[end].frame_index
                });
                if !caused {
                    return None;
                }

                let okizeme = classify_okizeme(
                    attacker,
                    attacker_epoch,
                    epoch,
                    wakeup,
                    features,
                    wakeup_frame,
                    frames,
                );
                // 起き上がりの前後で meter epoch が続いている場合だけ、結果まで
                // 確定したものとして扱う。
                let confidence = if attacker_epoch.get(wakeup).copied() == Some(epoch) {
                    EventConfidence::High
                } else {
                    EventConfidence::Medium
                };

                Some(KnockdownEvent {
                    side: down_side,
                    attacker: attacker_side,
                    frame: start_frame,
                    wakeup_frame,
                    setup_frames: setup_frames as u32,
                    okizeme,
                    confidence,
                    round_no,
                })
            }
            let event = event_from_run(
                start,
                end,
                epoch,
                last_index,
                n,
                down_side,
                attacker_side,
                attacker,
                attacker_epoch,
                features,
                contacts,
                rounds,
            );
            if let Some(event) = event {
                out.push(event);
            }
        }
    }
    out.sort_by_key(|event| (event.frame, event.side));
    out
}

/// 起き上がりのフレームに攻撃判定が乗っていれば持続当て。
/// 乗っていなくても直後に発生が始まっていれば攻めを継続したものとする。
fn classify_okizeme(
    attacker: &[MeterState],
    attacker_epoch: &[i32],
    epoch: i32,
    wakeup: usize,
    features: &[FrameFeatures],
    wakeup_frame: u32,
    frames: usize,
) -> OkizemeOutcome {
    if attacker_epoch.get(wakeup).copied() == Some(epoch) && attacker[wakeup] == MeterState::Active
    {
        return OkizemeOutcome::Meaty;
    }
    let pressured = (wakeup..frames)
        .take_while(|&frame| features[frame].frame_index <= wakeup_frame + OKIZEME_PRESSURE_WINDOW)
        .any(|frame| {
            attacker_epoch.get(frame).copied() == Some(epoch)
                && matches!(attacker[frame], MeterState::Startup | MeterState::Active)
        });
    if pressured {
        OkizemeOutcome::Pressured
    } else {
        OkizemeOutcome::Neutral
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::test_support::feat;

    #[test]
    fn neutral_okizeme_scans_through_the_last_valid_frame_only() {
        assert_eq!(
            classify_okizeme(
                &[MeterState::Free],
                &[0],
                0,
                0,
                &[feat(10, 1.0, 1.0)],
                10,
                1,
            ),
            OkizemeOutcome::Neutral
        );
    }

    #[test]
    fn a_qualifying_stun_run_at_the_series_end_has_no_wakeup_frame() {
        let length = 60;
        let features: Vec<_> = (0..length)
            .map(|frame| feat(frame as u32, 1.0, 1.0))
            .collect();
        let meters = [
            vec![MeterState::Stun; length],
            vec![MeterState::Free; length],
        ];
        let epochs = [vec![0; length], vec![0; length]];
        let contacts = [ContactEvent {
            frame: 0,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: false,
            round_no: 1,
        }];
        let rounds = [RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: length as u32 - 1,
            winner: None,
            p1_hp_end: 0.5,
            p2_hp_end: 1.0,
        }];

        assert!(extract_knockdowns(KnockdownInputs {
            features: &features,
            meter_state: &meters,
            meter_epoch: &epochs,
            contacts: &contacts,
            rounds: &rounds,
        })
        .is_empty());
    }
}
