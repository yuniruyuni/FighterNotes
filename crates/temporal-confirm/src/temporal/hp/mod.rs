//! HP 読み取り値を時間方向に確定するパイプライン。

use crate::frame_features::FrameFeatures;
use crate::round_start::FightMarker;

mod crash;
mod monotonic;
mod recoverable;
mod uncertainty;

use crash::reject_hp_crashes;
use monotonic::{enforce_monotonic, normalize_structural_full_runs, round_reset_frames};
use recoverable::restore_confirmed_recoveries;
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
    confirm_hp_impl(features, None);
}

/// `FIGHT` 画像で確定したラウンド境界を使って HP を確定する。
///
/// HP・drive の構造的満タン判定は遮蔽補正だけに使い、単調系列のリセット位置は
/// marker に限定する。これにより、試合中のバー誤読が新ラウンドを作らない。
#[allow(clippy::ptr_arg)]
pub fn confirm_hp_with_fight_markers(
    features: &mut Vec<FrameFeatures>,
    markers: &[FightMarker],
    own_side: &str,
) {
    confirm_hp_impl(features, Some((markers, RawHpSide::from_name(own_side))));
}

fn confirm_hp_impl(
    features: &mut [FrameFeatures],
    fight_context: Option<(&[FightMarker], RawHpSide)>,
) {
    let mut own: Vec<_> = features.iter().map(|feature| feature.own_hp).collect();
    let mut opponent: Vec<_> = features.iter().map(|feature| feature.opponent_hp).collect();
    let own_source: Vec<_> = features.iter().map(|feature| feature.own_hp).collect();
    let opponent_source: Vec<_> = features.iter().map(|feature| feature.opponent_hp).collect();
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

    normalize_structural_full_runs(features, &mut own, &mut opponent, &match_frames);
    let reset_frames = match fight_context {
        Some((markers, own_raw_side)) => {
            normalize_fight_openings(
                features,
                &mut own,
                &mut opponent,
                &match_frames,
                markers,
                own_raw_side,
            );
            fight_reset_frames(features, markers)
        }
        None => round_reset_frames(&own, &opponent, &match_frames),
    };
    restore_confirmed_recoveries(&mut own, &own_source, &match_frames, &reset_frames);
    restore_confirmed_recoveries(
        &mut opponent,
        &opponent_source,
        &match_frames,
        &reset_frames,
    );
    enforce_monotonic(&mut own, &reset_frames);
    enforce_monotonic(&mut opponent, &reset_frames);

    own_uncertainty.backward_fill(&mut own);
    opponent_uncertainty.backward_fill(&mut opponent);

    for ((feature, own_hp), opponent_hp) in features.iter_mut().zip(own).zip(opponent) {
        feature.own_hp = own_hp;
        feature.opponent_hp = opponent_hp;
    }
}

fn normalize_fight_openings(
    features: &[FrameFeatures],
    own: &mut [f32],
    opponent: &mut [f32],
    match_frames: &[bool],
    markers: &[FightMarker],
    own_raw_side: RawHpSide,
) {
    let opponent_raw_side = own_raw_side.opposite();
    for pair in markers.windows(2) {
        normalize_fight_marker(
            features,
            own,
            opponent,
            match_frames,
            &pair[0],
            feature_index(features, pair[1].first_frame),
            own_raw_side,
            opponent_raw_side,
        );
    }
    if let Some(marker) = markers.last() {
        normalize_fight_marker(
            features,
            own,
            opponent,
            match_frames,
            marker,
            features.len(),
            own_raw_side,
            opponent_raw_side,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn normalize_fight_marker(
    features: &[FrameFeatures],
    own: &mut [f32],
    opponent: &mut [f32],
    match_frames: &[bool],
    marker: &FightMarker,
    hard_end: usize,
    own_raw_side: RawHpSide,
    opponent_raw_side: RawHpSide,
) {
    let start = feature_index(features, marker.first_frame);
    let end = feature_index(features, marker.last_frame);
    promote_fight_opening_side(
        features,
        own,
        match_frames,
        start,
        end,
        hard_end,
        own_raw_side,
    );
    promote_fight_opening_side(
        features,
        opponent,
        match_frames,
        start,
        end,
        hard_end,
        opponent_raw_side,
    );
}

#[derive(Clone, Copy)]
enum RawHpSide {
    Left,
    Right,
}

impl RawHpSide {
    fn from_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case("p2") {
            Self::Right
        } else {
            Self::Left
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn reliable_value(self, feature: &FrameFeatures) -> Option<f32> {
        let (value, quality) = match self {
            Self::Left => (feature.left_hp_raw, feature.left_hp_raw_quality),
            Self::Right => (feature.right_hp_raw, feature.right_hp_raw_quality),
        };
        (quality < 0.5 && value >= 0.0).then_some(value)
    }
}

fn promote_fight_opening_side(
    features: &[FrameFeatures],
    values: &mut [f32],
    match_frames: &[bool],
    start: usize,
    end: usize,
    hard_end: usize,
    raw_side: RawHpSide,
) {
    if start >= values.len() || end < start {
        return;
    }
    let end = end.min(values.len() - 1);
    let mut samples: Vec<f32> = features[start..=end]
        .iter()
        .filter_map(|feature| raw_side.reliable_value(feature))
        .collect();
    values[start..=end].fill(1.0);
    if samples.is_empty() {
        return;
    }
    samples.sort_unstable_by(f32::total_cmp);
    // FIGHT の前半は遷移や文字エフェクトでバーが欠けることがあるため、
    // 中央値より上側の代表値を使う。低い前ラウンド値を基準にしない。
    let baseline = samples[samples.len() * 3 / 4];

    const OPENING_EDGE_JITTER: f32 = 0.02;
    const MAX_UNCERTAIN_RUN: usize = GAP_FILL;
    let mut index = end + 1;
    let mut uncertain_run = 0;
    while index < hard_end.min(values.len()) && match_frames[index] {
        match raw_side.reliable_value(&features[index]) {
            Some(raw) if raw >= baseline - OPENING_EDGE_JITTER => {
                uncertain_run = 0;
                values[index] = 1.0;
            }
            Some(_) => break,
            None if uncertain_run < MAX_UNCERTAIN_RUN => {
                uncertain_run += 1;
                values[index] = 1.0;
            }
            None => break,
        }
        index += 1;
    }
}

fn fight_reset_frames(features: &[FrameFeatures], markers: &[FightMarker]) -> Vec<bool> {
    let mut reset_at = vec![false; features.len()];
    for marker in markers {
        if let Some(reset) = reset_at.get_mut(feature_index(features, marker.first_frame)) {
            *reset = true;
        }
    }
    reset_at
}

fn feature_index(features: &[FrameFeatures], frame: u32) -> usize {
    features
        .binary_search_by_key(&frame, |feature| feature.frame_index)
        .unwrap_or_else(|index| index.min(features.len().saturating_sub(1)))
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
