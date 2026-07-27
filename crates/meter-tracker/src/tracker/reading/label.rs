use std::collections::HashMap;

use frame_meter::{RowObs, CELL_COUNT};

use crate::calibration::{
    LABEL_BLANK_BASE, LABEL_DECIDE_MARGIN, LABEL_DIGIT_BASE, LABEL_DIGIT_MIN,
};

use super::super::MeterTracker;

impl MeterTracker {
    pub(super) fn slab_by_label(
        &self,
        observation: &RowObs,
        cell: i64,
        side: &str,
    ) -> Option<String> {
        let digit_correlations = observation.digit_corr.as_ref()?;
        let (run, visible) = self.run_back_len(side);
        let run = run?;
        let with_current = visible + 1;
        if with_current >= 100 {
            return None;
        }

        let without_current = visible;
        let with_positions = digit_layout(
            &if with_current >= 4 {
                with_current.to_string()
            } else {
                String::new()
            },
            cell,
        );
        let without_positions = digit_layout(
            &if without_current >= 4 {
                without_current.to_string()
            } else {
                String::new()
            },
            (cell - 1).rem_euclid(CELL_COUNT as i64),
        );
        let with_score = label_layout_score(digit_correlations, &with_positions, cell);
        let without_score = label_layout_score(digit_correlations, &without_positions, cell);

        decide_label(
            run,
            with_score,
            has_label_evidence(digit_correlations, &with_positions),
            without_score,
            has_label_evidence(digit_correlations, &without_positions),
        )
    }
}

pub(crate) fn digit_layout(digits: &str, right: i64) -> HashMap<usize, char> {
    let digits: Vec<char> = digits.chars().collect();
    digits
        .iter()
        .enumerate()
        .map(|(index, &character)| {
            let position = (right - digits.len() as i64 + 1 + index as i64)
                .rem_euclid(CELL_COUNT as i64) as usize;
            (position, character)
        })
        .collect()
}

pub(crate) fn label_layout_score(
    digit_correlations: &[[f32; 10]],
    positions: &HashMap<usize, char>,
    cell: i64,
) -> f64 {
    [2i64, 1, 0]
        .into_iter()
        .filter_map(|offset| {
            let position = (cell - offset).rem_euclid(CELL_COUNT as i64) as usize;
            let correlation = digit_correlations.get(position)?;
            Some(if let Some(&character) = positions.get(&position) {
                let digit_index = character as usize - '0' as usize;
                correlation[digit_index] as f64 - LABEL_DIGIT_BASE
            } else {
                let maximum = correlation
                    .iter()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max);
                LABEL_BLANK_BASE - maximum as f64
            })
        })
        .sum()
}

pub(crate) fn has_label_evidence(
    digit_correlations: &[[f32; 10]],
    positions: &HashMap<usize, char>,
) -> bool {
    positions.iter().any(|(&position, &character)| {
        digit_correlations.get(position).is_some_and(|correlation| {
            let digit_index = character as usize - '0' as usize;
            (correlation[digit_index] as f64) >= LABEL_DIGIT_MIN
        })
    })
}

pub(crate) fn decide_label(
    run: String,
    with_score: f64,
    with_evidence: bool,
    without_score: f64,
    without_evidence: bool,
) -> Option<String> {
    if with_score - without_score >= LABEL_DECIDE_MARGIN && with_evidence {
        Some(run)
    } else if without_score - with_score >= LABEL_DECIDE_MARGIN && without_evidence {
        Some("empty".to_string())
    } else {
        None
    }
}
