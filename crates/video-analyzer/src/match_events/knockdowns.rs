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

use super::{
    round_of, ContactEvent, EventConfidence, KnockdownEvent, MeterState, OkizemeOutcome, RoundInfo,
    KNOCKDOWN_CAUSE_GRACE, KNOCKDOWN_MIN_SETUP, KNOCKDOWN_MIN_STUN, OKIZEME_PRESSURE_WINDOW,
};
use crate::frame_features::FrameFeatures;

pub(crate) struct KnockdownInputs<'a> {
    pub(crate) features: &'a [FrameFeatures],
    pub(crate) meter_state: &'a [Vec<MeterState>; 2],
    pub(crate) meter_epoch: &'a [Vec<i32>; 2],
    pub(crate) contacts: &'a [ContactEvent],
    pub(crate) rounds: &'a [RoundInfo],
}

pub(crate) fn extract_knockdowns(inputs: KnockdownInputs<'_>) -> Vec<KnockdownEvent> {
    let KnockdownInputs {
        features,
        meter_state,
        meter_epoch,
        contacts,
        rounds,
    } = inputs;
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();

    for down_index in 0..2usize {
        let down_side = down_index as u8 + 1;
        let attacker_side = 3 - down_side;
        let down = &meter_state[down_index];
        let attacker = &meter_state[1 - down_index];
        let down_epoch = &meter_epoch[down_index];
        let attacker_epoch = &meter_epoch[1 - down_index];

        let mut index = 0usize;
        while index < n {
            if down[index] != MeterState::Stun {
                index += 1;
                continue;
            }
            let start = index;
            // epoch を読めない区間は run を作れない。ここで走査位置を進めて
            // おかないと、後続の run 探索が同じ位置を再評価し続けて止まらない。
            let Some(epoch) = down_epoch.get(start).copied().filter(|epoch| *epoch >= 0) else {
                index += 1;
                continue;
            };
            while index < n
                && down[index] == MeterState::Stun
                && down_epoch.get(index).copied() == Some(epoch)
            {
                index += 1;
            }
            let end = index - 1;
            if end + 1 - start < KNOCKDOWN_MIN_STUN {
                continue;
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
                continue;
            }

            let start_frame = features[start].frame_index;
            // 起き上がりは stun 終端の次のフレーム。系列末尾なら結果を見ない。
            if end + 1 >= n {
                continue;
            }
            let wakeup = end + 1;
            let wakeup_frame = features[wakeup].frame_index;
            let Some(round_no) = round_of(rounds, start_frame) else {
                continue;
            };

            // ダウンさせたのが誰かは、原因のヒット接触から取る。
            let caused = contacts.iter().any(|contact| {
                contact.hit
                    && contact.victim == down_side
                    && contact.attacker == attacker_side
                    && contact.frame + KNOCKDOWN_CAUSE_GRACE >= start_frame
                    && contact.frame <= features[end].frame_index
            });
            if !caused {
                continue;
            }

            let okizeme = classify_okizeme(
                attacker,
                attacker_epoch,
                epoch,
                wakeup,
                features,
                wakeup_frame,
                n,
            );
            // 起き上がりの前後で meter epoch が続いている場合だけ、結果まで
            // 確定したものとして扱う。
            let confidence = if attacker_epoch.get(wakeup).copied() == Some(epoch) {
                EventConfidence::High
            } else {
                EventConfidence::Medium
            };

            out.push(KnockdownEvent {
                side: down_side,
                attacker: attacker_side,
                frame: start_frame,
                wakeup_frame,
                setup_frames: setup_frames as u32,
                okizeme,
                confidence,
                round_no,
            });
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
