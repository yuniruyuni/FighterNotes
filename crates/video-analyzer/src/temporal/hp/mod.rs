//! HP 読み取り値を時間方向に確定するパイプライン。

use crate::frame_features::FrameFeatures;

mod crash;
mod monotonic;
mod uncertainty;

use crash::reject_hp_crashes;
use monotonic::{enforce_monotonic, round_reset_frames};
use uncertainty::{forward_fill, UncertaintyWindow};

const GAP_FILL: usize = 10;

/// ラウンド開始とみなす「両者ほぼ全快」の閾値。
pub const FULL_HP: f32 = 0.985;
/// ラウンド開始とみなす全快状態の最小持続フレーム数。
pub const FULL_MIN_RUN: usize = 20;

/// HP を確定させる（own_hp / opponent_hp を上書き）。
///
/// 遮蔽区間の補間、画面遷移による急落の棄却、ラウンド内の単調減少、
/// ラウンド開始時のリセット、K.O. 後の後方補間を順番に適用する。
// Vec is retained as part of the public compatibility surface.
#[allow(clippy::ptr_arg)]
pub fn confirm_hp(features: &mut Vec<FrameFeatures>) {
    let mut own: Vec<_> = features.iter().map(|feature| feature.own_hp).collect();
    let mut opponent: Vec<_> = features.iter().map(|feature| feature.opponent_hp).collect();
    let match_frames: Vec<_> = features
        .iter()
        .map(|feature| feature.is_match_screen)
        .collect();

    let own_uncertainty = UncertaintyWindow::new(&own, GAP_FILL);
    let opponent_uncertainty = UncertaintyWindow::new(&opponent, GAP_FILL);
    own_uncertainty.obscure_neighbors(&mut own);
    opponent_uncertainty.obscure_neighbors(&mut opponent);
    forward_fill(&mut own);
    forward_fill(&mut opponent);

    reject_hp_crashes(&mut own, &match_frames);
    reject_hp_crashes(&mut opponent, &match_frames);

    let reset_frames = round_reset_frames(&own, &opponent, &match_frames);
    enforce_monotonic(&mut own, &reset_frames);
    enforce_monotonic(&mut opponent, &reset_frames);

    own_uncertainty.backward_fill(&mut own);
    opponent_uncertainty.backward_fill(&mut opponent);

    for ((feature, own_hp), opponent_hp) in features.iter_mut().zip(own).zip(opponent) {
        feature.own_hp = own_hp;
        feature.opponent_hp = opponent_hp;
    }
}

#[cfg(test)]
pub(super) const TEST_CRASH_CONFIRM: usize = crash::CRASH_CONFIRM;

#[cfg(test)]
pub(super) fn test_reject_hp_crashes(values: &mut [f32], match_frames: &[bool]) {
    crash::reject_hp_crashes(values, match_frames);
}

#[cfg(test)]
pub(super) fn test_expand_uncertain(uncertain: &[bool], gap_fill: usize) -> Vec<bool> {
    uncertainty::expand_uncertain(uncertain, gap_fill)
}

#[cfg(test)]
pub(super) fn test_round_reset_frames(
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
) -> Vec<bool> {
    monotonic::round_reset_frames(own, opponent, match_frames)
}

#[cfg(test)]
pub(super) fn test_enforce_monotonic(values: &mut [f32], reset_at: &[bool]) {
    monotonic::enforce_monotonic(values, reset_at);
}

#[cfg(test)]
pub(super) fn test_obscure_neighbors(source: &[f32], values: &mut [f32], gap_fill: usize) {
    uncertainty::UncertaintyWindow::new(source, gap_fill).obscure_neighbors(values);
}

#[cfg(test)]
pub(super) fn test_backward_fill(source: &[f32], values: &mut [f32], gap_fill: usize) {
    uncertainty::UncertaintyWindow::new(source, gap_fill).backward_fill(values);
}
