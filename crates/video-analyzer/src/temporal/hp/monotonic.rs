use super::{FULL_HP, FULL_MIN_RUN};
use crate::frame_features::FrameFeatures;

/// A stage-colored overlay can hide the internal white cap while leaving the
/// rest of a visually full HP bar intact. The observed false edge is around
/// 91%, so this threshold is only used together with round-transition and
/// full-drive evidence; it is never a general full-HP threshold.
const STRUCTURAL_FULL_HP: f32 = 0.90;
const FULL_DRIVE: f32 = 0.95;
const ROUND_GAP_MIN: usize = 20;
const ROUND_GAP_LOOKBACK: usize = 180;
const ROUND_RECOVERY_MIN: f32 = 0.08;
const OPENING_EDGE_JITTER: f32 = 0.015;

pub(super) fn round_reset_frames(
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
) -> Vec<bool> {
    let is_full =
        |index: usize| match_frames[index] && own[index] >= FULL_HP && opponent[index] >= FULL_HP;
    let mut reset_at = vec![false; own.len()];
    let mut run_start = None;

    for index in 0..own.len() {
        if is_full(index) {
            run_start.get_or_insert(index);
        } else if let Some(start) = run_start.take() {
            mark_reset(&mut reset_at, start, index);
        }
    }
    if let Some(start) = run_start {
        mark_reset(&mut reset_at, start, own.len());
    }

    reset_at
}

/// Promote a conservatively confirmed round-opening run to full HP.
///
/// A relaxed HP value is insufficient on its own: the run must last as long
/// as a normal full-health run, both drive gauges must be visibly full and
/// reliable, a sustained non-match transition must occur shortly beforehand,
/// and HP must have recovered materially from the previous match section.
/// These conditions keep ordinary 90% HP neutral situations inside a round
/// from becoming resets.
pub(super) fn normalize_structural_full_runs(
    features: &[FrameFeatures],
    own: &mut [f32],
    opponent: &mut [f32],
    match_frames: &[bool],
) {
    let mut index = 0;
    while index < own.len() {
        if !is_structural_full(features, own, opponent, match_frames, index) {
            index += 1;
            continue;
        }
        let start = index;
        while index < own.len() && is_structural_full(features, own, opponent, match_frames, index)
        {
            index += 1;
        }
        if index - start < FULL_MIN_RUN {
            continue;
        }
        let Some((gap_start, _gap_end)) = recent_non_match_gap(match_frames, start) else {
            continue;
        };
        if !has_round_recovery(own, opponent, match_frames, gap_start, start) {
            continue;
        }
        let own_baseline = own[start..index].iter().copied().fold(1.0, f32::min);
        let opponent_baseline = opponent[start..index].iter().copied().fold(1.0, f32::min);
        let own_end = promote_opening_side(own, match_frames, start, own_baseline);
        let opponent_end = promote_opening_side(opponent, match_frames, start, opponent_baseline);
        index = index.max(own_end).max(opponent_end);
    }
}

fn promote_opening_side(
    values: &mut [f32],
    match_frames: &[bool],
    start: usize,
    baseline: f32,
) -> usize {
    let mut end = start;
    while end < values.len() && match_frames[end] && values[end] >= baseline - OPENING_EDGE_JITTER {
        values[end] = 1.0;
        end += 1;
    }
    end
}

fn is_structural_full(
    features: &[FrameFeatures],
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
    index: usize,
) -> bool {
    let Some(feature) = features.get(index) else {
        return false;
    };
    match_frames.get(index) == Some(&true)
        && own
            .get(index)
            .is_some_and(|value| *value >= STRUCTURAL_FULL_HP)
        && opponent
            .get(index)
            .is_some_and(|value| *value >= STRUCTURAL_FULL_HP)
        && feature.left_drive_ratio >= FULL_DRIVE
        && feature.right_drive_ratio >= FULL_DRIVE
        && !feature.left_burnout
        && !feature.right_burnout
        && !feature.left_drive_uncertain
        && !feature.right_drive_uncertain
}

fn recent_non_match_gap(match_frames: &[bool], start: usize) -> Option<(usize, usize)> {
    let search_start = start.saturating_sub(ROUND_GAP_LOOKBACK);
    let mut latest = None;
    let mut index = search_start;
    while index < start {
        if match_frames[index] {
            index += 1;
            continue;
        }
        let gap_start = index;
        while index < start && !match_frames[index] {
            index += 1;
        }
        if index - gap_start >= ROUND_GAP_MIN {
            latest = Some((gap_start, index));
        }
    }
    latest
}

fn has_round_recovery(
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
    gap_start: usize,
    candidate_start: usize,
) -> bool {
    let candidate_min = own[candidate_start].min(opponent[candidate_start]);
    let prior_min = (0..gap_start)
        .rev()
        .filter(|&index| match_frames[index])
        .filter_map(|index| {
            let value = own[index].min(opponent[index]);
            (value >= 0.0).then_some(value)
        })
        .take(ROUND_GAP_LOOKBACK)
        .reduce(f32::min);

    prior_min.is_none_or(|previous| candidate_min >= previous + ROUND_RECOVERY_MIN)
}

fn mark_reset(reset_at: &mut [bool], start: usize, end: usize) {
    if end - start >= FULL_MIN_RUN {
        reset_at[start] = true;
    }
}

pub(super) fn enforce_monotonic(values: &mut [f32], reset_at: &[bool]) {
    let mut previous = None;
    for (index, value) in values.iter_mut().enumerate() {
        if reset_at[index] {
            previous = None;
        }
        if *value < 0.0 {
            continue;
        }
        if let Some(previous) = previous {
            *value = (*value).min(previous);
        }
        previous = Some(*value);
    }
}
