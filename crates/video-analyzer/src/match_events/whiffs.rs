//! 接触しなかった通常技の攻撃判定（空振り）の抽出。
//!
//! 差し合いの収支は「振った技が届いたか」「届かなかった技を狩られたか」で
//! 決まるが、既存の確反イベントは相手の後隙 run を起点にしており、
//! ガード起点を確認できない中距離の空振りそのものは残らない。
//!
//! 投げ・Drive Impact・無敵技はそれぞれ専用のイベントとカードが結果を
//! 追跡しているため、ここでは重複して数えない。弾は距離を取って撃つ行動が
//! 正常なので `ProjectileActive` も対象外とする。

use super::runs::{runs_of, MeterRun};
use super::{
    continuous_epoch, round_of, ContactEvent, DamageEvent, DriveImpactEvent, EventConfidence,
    MeterState, ReversalEvent, RoundInfo, ThrowActionEvent, WhiffEvent, WhiffOutcome,
    WHIFF_CONTACT_GRACE, WHIFF_PUNISH_WINDOW,
};
use crate::frame_features::FrameFeatures;

pub(crate) struct WhiffInputs<'a> {
    pub(crate) features: &'a [FrameFeatures],
    pub(crate) meter_state: &'a [Vec<MeterState>; 2],
    pub(crate) meter_epoch: &'a [Vec<i32>; 2],
    pub(crate) contacts: &'a [ContactEvent],
    pub(crate) damage: &'a [DamageEvent],
    pub(crate) throw_actions: &'a [ThrowActionEvent],
    pub(crate) drive_impacts: &'a [DriveImpactEvent],
    pub(crate) reversals: &'a [ReversalEvent],
    pub(crate) rounds: &'a [RoundInfo],
}

pub(crate) fn extract_whiffs(inputs: WhiffInputs<'_>) -> Vec<WhiffEvent> {
    let WhiffInputs {
        features,
        meter_state,
        meter_epoch,
        contacts,
        damage,
        throw_actions,
        drive_impacts,
        reversals,
        rounds,
    } = inputs;
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();

    for side_index in 0..2usize {
        let side = side_index as u8 + 1;
        let own = &meter_state[side_index];
        let own_epoch = &meter_epoch[side_index];

        for MeterRun { start, end, epoch } in runs_of(own, own_epoch, MeterState::Active) {
            let start_frame = features[start].frame_index;
            let end_frame = features[end].frame_index;
            let Some(round_no) = round_of(rounds, start_frame) else {
                continue;
            };

            // 接触が1つでもあれば空振りではない。ガードさせた場合も含む。
            let touched = contacts.iter().any(|contact| {
                contact.attacker == side
                    && contact.frame + WHIFF_CONTACT_GRACE >= start_frame
                    && contact.frame <= end_frame + WHIFF_CONTACT_GRACE
            });
            if touched {
                continue;
            }

            // 専用イベントが結果を追跡している行動を除外する。
            if is_tracked_elsewhere(
                side,
                start_frame,
                end_frame,
                throw_actions,
                drive_impacts,
                reversals,
            ) {
                continue;
            }

            // 硬直を狩られたか。空振りの終了より後の被接触だけを見る。
            let window_end = end_frame.saturating_add(WHIFF_PUNISH_WINDOW);
            let punish = contacts
                .iter()
                .filter(|contact| {
                    contact.victim == side
                        && contact.hit
                        && contact.frame > end_frame
                        && contact.frame <= window_end
                })
                .min_by_key(|contact| contact.frame);

            let (outcome, punished_frame, drop) = match punish {
                Some(contact) => {
                    let drop = damage
                        .iter()
                        .filter(|event| {
                            event.victim == side
                                && event.start_frame + WHIFF_CONTACT_GRACE >= contact.frame
                                && event.start_frame <= window_end
                        })
                        .map(|event| event.drop)
                        .fold(0.0_f32, f32::max);
                    (WhiffOutcome::Punished, Some(contact.frame), drop)
                }
                None => (WhiffOutcome::Unpunished, None, 0.0),
            };

            // 結果窓の間ずっと同じ meter epoch を追えた場合だけ、結果まで
            // 確定したものとして扱う。リセットをまたぐ観測は結び付けない。
            let window_index = idx_at_or_before(features, window_end).min(n - 1);
            let confidence = if continuous_epoch(own_epoch, start, window_index) == Some(epoch)
                && continuous_epoch(&meter_epoch[1 - side_index], start, window_index)
                    == Some(epoch)
            {
                EventConfidence::High
            } else {
                EventConfidence::Medium
            };

            out.push(WhiffEvent {
                side,
                frame: start_frame,
                end_frame,
                outcome,
                drop,
                punished_frame,
                confidence,
                round_no,
            });
        }
    }
    out.sort_by_key(|event| (event.frame, event.side));
    out
}

/// 投げ・Drive Impact・無敵技は専用イベントが結末を持つ。空振りとして
/// 二重に数えると、同じ被弾が複数のカードへ出る。
fn is_tracked_elsewhere(
    side: u8,
    start_frame: u32,
    end_frame: u32,
    throw_actions: &[ThrowActionEvent],
    drive_impacts: &[DriveImpactEvent],
    reversals: &[ReversalEvent],
) -> bool {
    let overlaps = |frame: u32| frame + WHIFF_CONTACT_GRACE >= start_frame && frame <= end_frame;
    throw_actions
        .iter()
        .any(|throw| throw.thrower == side && throw.active_frame.is_some_and(overlaps))
        || drive_impacts
            .iter()
            .any(|impact| impact.side == side && impact.active_frame.is_some_and(overlaps))
        || reversals.iter().any(|reversal| {
            reversal.side == side
                && reversal.frame + WHIFF_PUNISH_WINDOW >= start_frame
                && reversal.frame <= end_frame
        })
}

fn idx_at_or_before(features: &[FrameFeatures], frame: u32) -> usize {
    match features.binary_search_by_key(&frame, |feature| feature.frame_index) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    }
}
