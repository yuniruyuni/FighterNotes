//! Confirmed recoverable-HP restoration.
//!
//! Raw HP occasionally rises because of perception noise, so ordinary upward
//! movement must still be rejected by the monotonic pass.  Armor damage is
//! different: the bar stays below its previous level, then visibly recovers in
//! several reliable steps over a few seconds.  Only that strong temporal shape
//! is restored here, before permanent HP is made monotonic.

const MIN_DROP: f32 = 0.02;
const RETURN_TOLERANCE: f32 = 0.015;
const MIN_RETURN_RATIO: f32 = 0.70;
const MIN_RECOVERY_DELAY: usize = 30;
const MAX_RECOVERY_WINDOW: usize = 600;
const MIN_UPWARD_STEPS: usize = 3;
const MIN_UPWARD_STEP: f32 = 0.001;
const RECOVERY_CONFIRM_RUN: usize = 8;
const MAX_UNOBSERVED_EXTENSION: usize = 10;

pub(super) fn restore_confirmed_recoveries(
    values: &mut [f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
) {
    if values.is_empty()
        || source.len() != values.len()
        || match_frames.len() != values.len()
        || reset_at.len() != values.len()
    {
        return;
    }

    let mut previous = None;
    let mut index = 0;
    while index < values.len() {
        if reset_at[index] {
            previous = None;
        }
        let value = values[index];
        if value < 0.0 || !match_frames[index] {
            index += 1;
            continue;
        }
        let Some(baseline) = previous else {
            previous = Some(value);
            index += 1;
            continue;
        };

        if baseline - value < MIN_DROP {
            previous = Some(baseline.min(value));
            index += 1;
            continue;
        }

        if let Some(end) =
            confirmed_recovery_end(values, source, match_frames, reset_at, index, baseline)
        {
            for value in &mut values[index..=end] {
                if *value >= 0.0 {
                    *value = baseline;
                }
            }
            previous = Some(baseline);
            index = end + 1;
        } else {
            previous = Some(value);
            index += 1;
        }
    }
}

fn confirmed_recovery_end(
    values: &[f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
    drop_start: usize,
    baseline: f32,
) -> Option<usize> {
    let hard_end = drop_start
        .saturating_add(MAX_RECOVERY_WINDOW)
        .min(values.len().saturating_sub(1));
    let mut low = reliable_source(source, drop_start).unwrap_or(values[drop_start]);
    let mut previous_observed = None;
    let mut upward_steps = 0;
    let mut return_run = 0;

    for index in drop_start..=hard_end {
        if (index > drop_start && reset_at[index]) || !match_frames[index] {
            break;
        }
        let Some(value) = reliable_source(source, index) else {
            return_run = 0;
            continue;
        };

        low = low.min(value);
        if previous_observed.is_some_and(|previous| value >= previous + MIN_UPWARD_STEP) {
            upward_steps += 1;
        }
        previous_observed = Some(value);

        let drop = baseline - low;
        let returned = value - low;
        let near_baseline = baseline - value <= RETURN_TOLERANCE;
        let enough_return = drop >= MIN_DROP && returned >= drop * MIN_RETURN_RATIO;
        if index - drop_start >= MIN_RECOVERY_DELAY
            && near_baseline
            && enough_return
            && upward_steps >= MIN_UPWARD_STEPS
        {
            return_run += 1;
            if return_run >= RECOVERY_CONFIRM_RUN {
                return Some(extend_recovered_level(
                    values,
                    source,
                    match_frames,
                    reset_at,
                    index,
                    baseline,
                ));
            }
        } else {
            return_run = 0;
        }
    }
    None
}

fn extend_recovered_level(
    values: &[f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
    confirmed_at: usize,
    baseline: f32,
) -> usize {
    let mut end = confirmed_at;
    let mut unobserved_run = 0;
    for index in confirmed_at + 1..values.len() {
        if reset_at[index] || !match_frames[index] {
            break;
        }
        let Some(value) = reliable_source(source, index) else {
            unobserved_run += 1;
            if unobserved_run > MAX_UNOBSERVED_EXTENSION {
                break;
            }
            end = index;
            continue;
        };
        unobserved_run = 0;
        if value < baseline - RETURN_TOLERANCE {
            break;
        }
        end = index;
    }
    end
}

fn reliable_source(source: &[f32], index: usize) -> Option<f32> {
    source.get(index).copied().filter(|value| *value >= 0.0)
}
