use std::collections::HashSet;

use frame_meter::{RowObs, CELL_COUNT};

use crate::calibration::{BLACKISH_V_MAX, CELL_COUNT_I64, DIFF_V_MIN, DIFF_WF_MIN};

use super::MeterTracker;

impl MeterTracker {
    pub(crate) fn changed_cells(&self, left: &RowObs, right: &RowObs) -> HashSet<i64> {
        let Some(previous) = self.previous.as_ref() else {
            return HashSet::new();
        };
        let mut changed = HashSet::new();
        for (observation, previous) in [(left, &previous.0), (right, &previous.1)] {
            for index in 0..CELL_COUNT {
                let value_changed = (observation.v[index] - previous.v[index]).abs() > DIFF_V_MIN;
                let white_fraction_changed =
                    (observation.wf[index] - previous.wf[index]).abs() > DIFF_WF_MIN;
                if value_changed || white_fraction_changed {
                    let classification_changed = observation.states[index]
                        != previous.states[index]
                        || observation.bright[index] != previous.bright[index];
                    if classification_changed {
                        changed.insert(index as i64);
                    }
                }
            }
        }
        changed
    }

    pub(crate) fn wipe_count(&self, left: &RowObs, right: &RowObs) -> i64 {
        let Some(previous) = self.previous.as_ref() else {
            return 0;
        };
        [(left, &previous.0), (right, &previous.1)]
            .into_iter()
            .map(|(observation, previous)| {
                observation
                    .v
                    .iter()
                    .zip(&previous.v)
                    .filter(|(value, previous)| **previous > 70.0 && **value < 25.0)
                    .count() as i64
            })
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn select_edge(&self, left: i32, right: i32) -> i64 {
        let candidates: Vec<i64> = [left, right]
            .into_iter()
            .filter(|&edge| edge >= 0)
            .map(i64::from)
            .collect();
        if candidates.is_empty() {
            return -1;
        }
        let Some(absolute) = self.absolute_frame else {
            return *candidates.iter().max().expect("edge candidate");
        };
        let cell = absolute;
        let rank = |edge: i64| -> (i32, i64) {
            let delta = Self::circ_delta(edge, cell);
            if delta == 1 {
                (0, 0)
            } else if delta == 0 {
                (1, 0)
            } else if delta.is_positive() {
                (2, delta)
            } else {
                (3, -delta)
            }
        };
        *candidates
            .iter()
            .min_by_key(|&&edge| rank(edge))
            .expect("edge candidate")
    }

    pub(crate) fn circ_delta(left: i64, right: i64) -> i64 {
        let delta = (left - right).rem_euclid(CELL_COUNT_I64);
        if delta <= CELL_COUNT_I64 / 2 {
            delta
        } else {
            delta - CELL_COUNT_I64
        }
    }

    pub(crate) fn near_front(changed: &HashSet<i64>, predicted: i64) -> bool {
        changed.iter().any(|&cell| {
            let delta = Self::circ_delta(cell, predicted);
            (0..=4).contains(&delta)
        })
    }

    pub(crate) fn all_blackish(observation: &RowObs) -> bool {
        observation.v.iter().all(|&value| value < BLACKISH_V_MAX)
    }
}
