use crate::constants::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W, WHITE_V};

use super::source::RowPixels;

#[derive(Clone, Copy)]
pub(crate) struct CellBounds {
    pub(crate) x1: usize,
    pub(crate) x2: usize,
}

impl CellBounds {
    pub(crate) fn width(self) -> usize {
        self.x2 - self.x1
    }
}

pub(crate) fn cell_bounds(row_width: usize, index: usize) -> Option<CellBounds> {
    let cell_x1 = row_width * index / CELL_COUNT;
    let cell_x2 = row_width * (index + 1) / CELL_COUNT;
    let trim_x = ((cell_x2 - cell_x1) / 8).max(1);
    let x1 = cell_x1 + trim_x;
    let x2 = cell_x2.saturating_sub(trim_x).max(x1 + 1).min(row_width);
    (x1 < x2).then_some(CellBounds { x1, x2 })
}

pub(crate) fn min_cell_patch_width(row_width: usize) -> usize {
    (0..CELL_COUNT)
        .map(|index| cell_bounds(row_width, index).map(CellBounds::width))
        .collect::<Option<Vec<_>>>()
        .and_then(|widths| widths.into_iter().min())
        .unwrap_or(0)
}

pub(crate) fn mean_value(pixels: &RowPixels, bounds: CellBounds) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    for row in pixels.trim_y..pixels.height - pixels.trim_y {
        for column in bounds.x1..bounds.x2 {
            sum += pixels.value[row * pixels.width + column];
            count += 1;
        }
    }
    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}

pub(crate) fn write_column_means(output: &mut [f32], pixels: &RowPixels, bounds: CellBounds) {
    for (column, output) in output.iter_mut().enumerate() {
        let sum = (pixels.trim_y..pixels.height - pixels.trim_y)
            .map(|row| pixels.value[row * pixels.width + bounds.x1 + column])
            .sum::<f32>();
        *output = sum / pixels.patch_height as f32;
    }
}

pub(crate) fn write_digit_patch(
    output: &mut [f32],
    pixels: &RowPixels,
    bounds: CellBounds,
) -> bool {
    if pixels.patch_height < DIGIT_TEMPLATE_H || bounds.width() < DIGIT_TEMPLATE_W {
        return false;
    }
    let source_rows = pixels
        .value
        .chunks_exact(pixels.width)
        .skip(pixels.trim_y)
        .take(DIGIT_TEMPLATE_H);
    for (output_row, source_row) in output.chunks_exact_mut(DIGIT_TEMPLATE_W).zip(source_rows) {
        output_row.copy_from_slice(&source_row[bounds.x1..][..DIGIT_TEMPLATE_W]);
    }
    normalize(output);
    true
}

pub(crate) fn white_row_fraction(pixels: &RowPixels, bounds: CellBounds) -> f32 {
    let mut white_rows = 0;
    for local_row in 0..pixels.patch_height {
        let row = pixels.trim_y + local_row;
        let count = bounds.width();
        let mean_value = (bounds.x1..bounds.x2)
            .map(|column| pixels.value[row * pixels.width + column])
            .sum::<f32>()
            / count as f32;
        let mean_saturation = (bounds.x1..bounds.x2)
            .map(|column| pixels.saturation[row * pixels.width + column])
            .sum::<f32>()
            / count as f32;
        if mean_value > WHITE_V && mean_saturation < 30.0 {
            white_rows += 1;
        }
    }
    white_rows as f32 / pixels.patch_height as f32
}

fn normalize(patch: &mut [f32]) {
    let mean = patch.iter().sum::<f32>() / patch.len() as f32;
    let variance = patch
        .iter()
        .map(|&value| {
            let centered = value - mean;
            centered * centered
        })
        .sum::<f32>()
        / patch.len() as f32;
    let denominator = variance.sqrt().max(1.0);
    for value in patch {
        *value = (*value - mean) / denominator;
    }
}
