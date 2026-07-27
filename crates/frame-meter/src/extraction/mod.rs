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

use crate::constants::{
    LEFT_ROW_Y1, LEFT_ROW_Y2, METER_STRIP_Y, RIGHT_ROW_Y1, RIGHT_ROW_Y2, STRIPE_REGION1_ROWS,
    STRIPE_REGION2_ROWS,
};
use crate::model::RowObs;

use source::RowSource;

/// Extracts both players' frame-meter observations from a full RGBA frame.
pub fn extract_row_obs(rgba: &[u8], width: u32, height: u32) -> (RowObs, RowObs) {
    extract_rows(RowSource::new(rgba, width, height, 0))
}

/// Extracts observations from the minimal frame-meter strip at `METER_STRIP_Y`.
pub fn extract_row_obs_from_strip(
    meter_strip: &[u8],
    full_width: u32,
    full_height: u32,
) -> (RowObs, RowObs) {
    let scale_y = full_height as f32 / 1080.0;
    let strip_y = (METER_STRIP_Y as f32 * scale_y) as i32;
    extract_rows(RowSource::new(
        meter_strip,
        full_width,
        full_height,
        strip_y,
    ))
}

fn extract_rows(source: RowSource<'_>) -> (RowObs, RowObs) {
    (
        extract_row(&source, LEFT_ROW_Y1, LEFT_ROW_Y2),
        extract_row(&source, RIGHT_ROW_Y1, RIGHT_ROW_Y2),
    )
}

fn extract_row(source: &RowSource<'_>, y1: i32, y2: i32) -> RowObs {
    source
        .read_row(y1, y2, STRIPE_REGION1_ROWS, STRIPE_REGION2_ROWS)
        .as_ref()
        .map(cells::extract)
        .unwrap_or_else(RowObs::empty)
}
