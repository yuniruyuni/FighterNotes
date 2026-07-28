mod cell;
mod cells;
mod metrics;
mod source;

#[cfg(test)]
#[path = "../tests/extraction_cell.rs"]
mod cell_tests;
#[cfg(test)]
#[path = "../tests/extraction_cells.rs"]
mod cells_tests;
#[cfg(test)]
#[path = "../tests/extraction_metrics.rs"]
mod metrics_tests;
#[cfg(test)]
#[path = "../tests/extraction_pipeline.rs"]
mod pipeline_tests;
#[cfg(test)]
#[path = "../tests/extraction_source.rs"]
mod source_tests;

use crate::color::QuantizedModeScratch;
use crate::constants::{
    LEFT_ROW_Y1, LEFT_ROW_Y2, METER_STRIP_Y, RIGHT_ROW_Y1, RIGHT_ROW_Y2, STRIPE_REGION1_ROWS,
    STRIPE_REGION2_ROWS,
};
use crate::model::RowObs;

use source::RowSource;

enum DigitSelection {
    Full,
    Tracker(Option<(usize, usize)>),
}

/// Extracts both players' frame-meter observations from a full RGBA frame.
pub fn extract_row_obs(rgba: &[u8], width: u32, height: u32) -> (RowObs, RowObs) {
    extract_rows(RowSource::new(rgba, width, height, 0), DigitSelection::Full)
}

/// Extracts observations from the minimal frame-meter strip at `METER_STRIP_Y`.
pub fn extract_row_obs_from_strip(
    meter_strip: &[u8],
    full_width: u32,
    full_height: u32,
) -> (RowObs, RowObs) {
    let scale_y = full_height as f32 / 1080.0;
    let strip_y = (METER_STRIP_Y as f32 * scale_y) as i32;
    extract_rows(
        RowSource::new(meter_strip, full_width, full_height, strip_y),
        DigitSelection::Full,
    )
}

/// Extracts observations while limiting digit-template correlation to cells
/// that the meter tracker can consume on this frame. Frames without a tracker
/// hint or an observed cursor edge omit digit scoring entirely.
pub fn extract_row_obs_from_strip_with_digit_hint(
    meter_strip: &[u8],
    full_width: u32,
    full_height: u32,
    digit_hint: Option<(usize, usize)>,
) -> (RowObs, RowObs) {
    let scale_y = full_height as f32 / 1080.0;
    let strip_y = (METER_STRIP_Y as f32 * scale_y) as i32;
    extract_rows(
        RowSource::new(meter_strip, full_width, full_height, strip_y),
        DigitSelection::Tracker(digit_hint),
    )
}

fn extract_rows(source: RowSource<'_>, digit_selection: DigitSelection) -> (RowObs, RowObs) {
    let mut color_scratch = QuantizedModeScratch::new();
    let left = extract_row_parts(&source, LEFT_ROW_Y1, LEFT_ROW_Y2, &mut color_scratch);
    let right = extract_row_parts(&source, RIGHT_ROW_Y1, RIGHT_ROW_Y2, &mut color_scratch);
    let digit_hint = match digit_selection {
        DigitSelection::Full => return (left.finish_full(), right.finish_full()),
        DigitSelection::Tracker(digit_hint) => digit_hint,
    };

    let left_edge = left.observation.fresh_edge;
    let right_edge = right.observation.fresh_edge;
    let mut valid = [0u64; 2];
    if let Some((current_cell, lookback)) = digit_hint {
        add_digit_window(&mut valid, current_cell, lookback);
        add_digit_window(&mut valid, current_cell + 1, lookback);
    }
    for edge in [left_edge, right_edge] {
        if edge >= 0 {
            add_digit_window(
                &mut valid,
                edge as usize,
                digit_hint.map_or(crate::constants::CELL_COUNT - 1, |(_, lookback)| lookback),
            );
        }
    }
    if valid == [0; 2] {
        return (left.finish_without_digits(), right.finish_without_digits());
    }
    (left.finish_sparse(valid), right.finish_sparse(valid))
}

fn extract_row_parts(
    source: &RowSource<'_>,
    y1: i32,
    y2: i32,
    color_scratch: &mut QuantizedModeScratch,
) -> cells::CellExtraction {
    source
        .read_row(y1, y2, STRIPE_REGION1_ROWS, STRIPE_REGION2_ROWS)
        .map(|pixels| cells::extract_parts(pixels, color_scratch))
        .unwrap_or_else(cells::CellExtraction::empty)
}

fn add_digit_window(valid: &mut [u64; 2], center: usize, lookback: usize) {
    let center = center % crate::constants::CELL_COUNT;
    for offset in 0..=lookback.min(crate::constants::CELL_COUNT - 1) {
        let index = (center + crate::constants::CELL_COUNT - offset) % crate::constants::CELL_COUNT;
        valid[index / 64] |= 1 << (index % 64);
    }
}
