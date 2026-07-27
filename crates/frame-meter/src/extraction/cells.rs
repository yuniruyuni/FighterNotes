use crate::constants::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W, HIGHLIGHT_V_MIN};
use crate::digits::digit_correlations;
use crate::edge::fresh_color_edge;
use crate::model::{BrightClass, CellState, RowObs};

use super::cell;
use super::metrics::{
    cell_bounds, mean_value, min_cell_patch_width, white_row_fraction, write_column_means,
    write_digit_patch,
};
use super::source::RowPixels;

pub(crate) fn extract(pixels: &RowPixels) -> RowObs {
    let column_width = min_cell_patch_width(pixels.width);
    let mut columns = vec![0.0; CELL_COUNT * column_width];
    let digit_stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    let mut digit_patches = vec![0.0; CELL_COUNT * digit_stride];
    let mut digit_patch_count = 0;
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

        let value = mean_value(pixels, bounds);
        values.push(value);
        let start = index * column_width;
        write_column_means(&mut columns[start..start + column_width], pixels, bounds);
        let digit_start = index * digit_stride;
        if write_digit_patch(
            &mut digit_patches[digit_start..digit_start + digit_stride],
            pixels,
            bounds,
        ) {
            digit_patch_count += 1;
        }
        white_fractions.push(white_row_fraction(pixels, bounds));

        let classified = cell::classify(pixels, bounds, value, &mut region1, &mut region2);
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
    let digit_corr = if digit_patch_count == CELL_COUNT {
        digit_correlations(&digit_patches)
    } else {
        None
    };

    RowObs {
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
        digit_corr,
        slab_pos,
        slab_state: None,
    }
}
