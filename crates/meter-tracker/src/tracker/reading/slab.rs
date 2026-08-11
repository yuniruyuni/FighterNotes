use frame_meter::RowObs;

use crate::calibration::{CELL_COUNT_I64, SLAB_SLIDE_MAD_MAX, SLAB_STATIC_MAD_MAX};

use super::{is_information, MeterTracker};

impl MeterTracker {
    pub(crate) fn resolve_slab(
        &self,
        observation: &RowObs,
        previous: Option<&RowObs>,
        cell: i64,
        side: &str,
    ) -> String {
        if let Some(decided) = self.slab_by_label(observation, cell, side) {
            return decided;
        }
        let Some(previous) = previous else {
            return "other".to_string();
        };
        let (Some(columns), Some(previous_columns)) =
            (observation.cols.as_ref(), previous.cols.as_ref())
        else {
            return String::from("other");
        };

        let slide_difference = compare_columns(
            columns,
            observation.cols_w,
            previous_columns,
            previous.cols_w,
            cell,
            &[2, 1, 0],
            &[3, 2, 1],
        );
        if slide_difference <= SLAB_SLIDE_MAD_MAX {
            return "empty".to_string();
        }

        let static_difference = compare_columns(
            columns,
            observation.cols_w,
            previous_columns,
            previous.cols_w,
            cell,
            &[2, 1],
            &[2, 1],
        );
        if static_difference <= SLAB_STATIC_MAD_MAX {
            return "empty".to_string();
        }

        if let Some(run) = self.run_state(side) {
            if is_information(&run) {
                return run;
            }
        }
        let previous_cell = (cell - 1).rem_euclid(CELL_COUNT_I64) as usize;
        let previous_state = observation.states[previous_cell].as_str();
        if is_information(previous_state) {
            previous_state.to_string()
        } else {
            "other".to_string()
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compare_columns(
    current: &[f32],
    current_width: usize,
    previous: &[f32],
    previous_width: usize,
    cell: i64,
    current_offsets: &[i64],
    previous_offsets: &[i64],
) -> f64 {
    let mut current_values = Vec::new();
    let mut previous_values = Vec::new();
    for (&current_offset, &previous_offset) in current_offsets.iter().zip(previous_offsets.iter()) {
        let current_cell = (cell - current_offset).rem_euclid(CELL_COUNT_I64) as usize;
        let previous_cell = (cell - previous_offset).rem_euclid(CELL_COUNT_I64) as usize;
        let width = current_width.min(previous_width);
        for index in 0..width {
            current_values.push(current[current_cell * current_width + index]);
            previous_values.push(previous[previous_cell * previous_width + index]);
        }
    }
    if current_values.is_empty() || current_values.len() != previous_values.len() {
        f64::MAX
    } else {
        mean_absolute_difference(&current_values, &previous_values)
    }
}

pub(crate) fn mean_absolute_difference(left: &[f32], right: &[f32]) -> f64 {
    if left.is_empty() {
        return 0.0;
    }
    left.iter()
        .zip(right)
        .map(|(&left, &right)| (left - right).abs() as f64)
        .sum::<f64>()
        / left.len() as f64
}
