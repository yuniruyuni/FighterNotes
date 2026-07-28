pub(crate) mod label;
pub(crate) mod slab;

use frame_meter::{RowObs, CELL_COUNT};

use crate::calibration::LABEL_DIGIT_MIN;

use super::{MeterTracker, ReadEntry};

impl MeterTracker {
    pub(super) fn run_state(&self, side: &str) -> Option<String> {
        let absolute = self.absolute_frame?;
        let reads = self.reads.get(side)?;
        for back in 1..=3 {
            let target = absolute - back;
            if let Some(read) = reads.get(&target) {
                if is_information(&read.0) {
                    return Some(read.0.clone());
                }
            }
        }
        None
    }

    pub(super) fn run_back_len(&self, side: &str) -> (Option<String>, i64) {
        let Some(absolute) = self.absolute_frame else {
            return (None, 0);
        };
        let reads = self.reads.get(side).expect("side reads");
        let emitted = self.emitted.get(side).expect("side emitted reads");
        let mut state = None;
        let mut length = 0;

        for target in (0..absolute).rev() {
            let candidate = reads
                .get(&target)
                .map(|read| read.0.clone())
                .or_else(|| emitted.get(&target).cloned());
            match candidate {
                None => break,
                Some(candidate) if !is_information(&candidate) => break,
                Some(candidate) => {
                    if state.is_none() {
                        state = Some(candidate.clone());
                    } else if state.as_ref() != Some(&candidate) {
                        break;
                    }
                    length += 1;
                }
            }
        }
        (state, length)
    }

    pub(crate) fn digit_covered(observation: &RowObs, cell: i64) -> bool {
        let cell = cell.rem_euclid(CELL_COUNT as i64) as usize;
        observation
            .digit_correlation(cell)
            .is_some_and(|correlation| {
                (correlation
                    .iter()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max) as f64)
                    >= LABEL_DIGIT_MIN
            })
    }

    pub(crate) fn better_read(
        current: Option<&ReadEntry>,
        state: &str,
        confidence: f64,
        covered: bool,
    ) -> bool {
        let Some((current_state, current_confidence, current_covered)) = current else {
            return true;
        };
        let current_covered = *current_covered;
        let new_information = is_read_information(state);
        let current_information = is_read_information(current_state);
        if new_information {
            if current_information && covered && !current_covered {
                return false;
            }
            confidence > *current_confidence || !current_information
        } else {
            !current_information && confidence > *current_confidence
        }
    }

    pub(crate) fn dim_read(observation: &RowObs, position: i64) -> Option<String> {
        let state = observation.states[position as usize].as_str();
        is_information(state).then(|| state.to_string())
    }
}

pub(super) fn is_information(state: &str) -> bool {
    state != "empty" && state != "other" && state != "unknown"
}

pub(crate) fn is_read_information(state: &str) -> bool {
    state != "empty" && state != "other"
}
