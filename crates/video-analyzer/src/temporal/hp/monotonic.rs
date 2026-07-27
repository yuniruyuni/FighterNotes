use super::{FULL_HP, FULL_MIN_RUN};

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
