use crate::color::QuantizedModeScratch;
use crate::constants::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W, HIGHLIGHT_V_MIN};
use crate::digits::{digit_correlations, digit_correlations_for_compact_patches};
use crate::edge::fresh_color_edge;
use crate::model::{BrightClass, CellState, RowObs};

use super::cell;
use super::metrics::{
    cell_bounds, mean_value, min_cell_patch_width, white_row_fraction, write_column_means,
    write_digit_patch,
};
use super::source::RowPixels;

pub(crate) struct CellExtraction {
    pub(crate) observation: RowObs,
    pixels: Option<RowPixels>,
}

impl CellExtraction {
    pub(crate) fn empty() -> Self {
        Self {
            observation: RowObs::empty(),
            pixels: None,
        }
    }

    pub(crate) fn finish_full(mut self) -> RowObs {
        let cells = (0..CELL_COUNT).collect::<Vec<_>>();
        if let Some(digit_patches) = self.digit_patches(&cells) {
            self.observation.digit_corr = digit_correlations(&digit_patches);
        }
        self.observation
    }

    pub(crate) fn finish_sparse(mut self, valid: [u64; 2]) -> RowObs {
        let cells = (0..CELL_COUNT)
            .filter(|&index| {
                let word = index / 64;
                let bit = index % 64;
                valid[word] & (1 << bit) != 0
            })
            .collect::<Vec<_>>();
        if let Some(digit_patches) = self.digit_patches(&cells) {
            self.observation.digit_corr =
                digit_correlations_for_compact_patches(&digit_patches, &cells);
        }
        self.observation
    }

    pub(crate) fn finish_without_digits(self) -> RowObs {
        self.observation
    }

    fn digit_patches(&self, cells: &[usize]) -> Option<Vec<f32>> {
        if cells.is_empty() {
            return None;
        }
        let pixels = self.pixels.as_ref()?;
        if pixels.patch_height < DIGIT_TEMPLATE_H || self.observation.cols_w < DIGIT_TEMPLATE_W {
            return None;
        }

        let digit_stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
        let mut digit_patches = vec![0.0; cells.len() * digit_stride];
        for (patch_index, &cell_index) in cells.iter().enumerate() {
            let bounds = cell_bounds(pixels.width, cell_index)?;
            let start = patch_index * digit_stride;
            if !write_digit_patch(
                &mut digit_patches[start..start + digit_stride],
                pixels,
                bounds,
            ) {
                return None;
            }
        }
        Some(digit_patches)
    }
}

#[cfg(test)]
pub(crate) fn extract(pixels: RowPixels) -> RowObs {
    extract_parts(pixels, &mut QuantizedModeScratch::new()).finish_full()
}

pub(crate) fn extract_parts(
    pixels: RowPixels,
    color_scratch: &mut QuantizedModeScratch,
) -> CellExtraction {
    let column_width = min_cell_patch_width(pixels.width);
    let mut columns = vec![0.0; CELL_COUNT * column_width];
    let mut region1 = Vec::new();
    let mut region2 = Vec::new();

    let mut values = Vec::with_capacity(CELL_COUNT);
    let mut white_fractions = Vec::with_capacity(CELL_COUNT);
    let mut colors = Vec::with_capacity(CELL_COUNT);
    let mut stripes = Vec::with_capacity(CELL_COUNT);
    let mut states = Vec::with_capacity(CELL_COUNT);
    let mut brightness = Vec::with_capacity(CELL_COUNT);
    let mut rescued = Vec::with_capacity(CELL_COUNT);
    let mut quality = Vec::with_capacity(CELL_COUNT);

    for index in 0..CELL_COUNT {
        let Some(bounds) = cell_bounds(pixels.width, index) else {
            values.push(0.0);
            white_fractions.push(0.0);
            colors.push([0.0; 3]);
            stripes.push(false);
            states.push(CellState::Empty);
            brightness.push(BrightClass::None_);
            rescued.push(false);
            quality.push(0.0);
            continue;
        };

        let value = mean_value(&pixels, bounds);
        values.push(value);
        let start = index * column_width;
        write_column_means(&mut columns[start..start + column_width], &pixels, bounds);
        white_fractions.push(white_row_fraction(&pixels, bounds));

        let classified = cell::classify(
            &pixels,
            bounds,
            value,
            &mut region1,
            &mut region2,
            color_scratch,
        );
        stripes.push(classified.state.is_stripe());
        colors.push(classified.bgr);
        brightness.push(classified.bright);
        rescued.push(classified.rescued);
        quality.push(classified.quality);
        states.push(classified.state);
    }

    let slab_pos = states
        .iter()
        .enumerate()
        .rev()
        .find(|(index, state)| **state == CellState::Other && values[*index] >= HIGHLIGHT_V_MIN)
        .map_or(-1, |(index, _)| index as i32);
    let fresh_edge = fresh_color_edge(&values, &white_fractions, &states, &brightness);
    CellExtraction {
        observation: RowObs {
            v: values,
            wf: white_fractions,
            states,
            bright: brightness,
            fresh_edge,
            bgr: colors,
            stripe: stripes,
            cols: (column_width > 0).then_some(columns),
            cols_w: column_width,
            rescued,
            quality,
            digit_corr: None,
            slab_pos,
            slab_state: None,
        },
        pixels: Some(pixels),
    }
}
