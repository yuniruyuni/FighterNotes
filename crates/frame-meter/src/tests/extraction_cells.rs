use crate::constants::{STRIPE_REGION1_ROWS, STRIPE_REGION2_ROWS};
use crate::extraction::cells;
use crate::extraction::source::RowPixels;
use crate::{CellState, CELL_COUNT, HIGHLIGHT_V_MIN};

fn uniform_pixels(width: usize, bgr: [u8; 3], value: f32) -> RowPixels {
    let height = 38;
    let size = width * height;
    RowPixels {
        width,
        height,
        trim_y: 6,
        patch_height: 26,
        region1_rows: STRIPE_REGION1_ROWS.to_vec(),
        region2_rows: STRIPE_REGION2_ROWS.to_vec(),
        bgr: vec![bgr; size],
        value: vec![value; size],
        saturation: vec![0.0; size],
    }
}

#[test]
fn extraction_populates_columns_digits_and_classification_for_every_cell() {
    let row = cells::extract(&uniform_pixels(1200, [146, 201, 19], 201.0));

    assert_eq!(row.states, vec![CellState::Counter; CELL_COUNT]);
    assert_eq!(row.cols_w, 13);
    assert_eq!(row.cols, Some(vec![201.0; CELL_COUNT * 13]));
    assert_eq!(row.digit_corr.as_ref().map(Vec::len), Some(CELL_COUNT));
    assert_eq!(row.slab_pos, -1);
}

#[test]
fn extraction_uses_none_for_columns_and_digits_when_cells_do_not_fit() {
    let row = cells::extract(&uniform_pixels(1, [23, 20, 23], 23.0));

    assert_eq!(row.states.len(), CELL_COUNT);
    assert_eq!(row.cols_w, 0);
    assert_eq!(row.cols, None);
    assert_eq!(row.digit_corr, None);
}

#[test]
fn slab_requires_other_state_and_highlight_value() {
    let highlighted = cells::extract(&uniform_pixels(1200, [23, 20, 23], HIGHLIGHT_V_MIN));
    assert_eq!(highlighted.slab_pos, 79);

    let dim_other = cells::extract(&uniform_pixels(
        1200,
        [236, 233, 233],
        HIGHLIGHT_V_MIN - 1.0,
    ));
    assert_eq!(dim_other.slab_pos, -1);

    let colored = cells::extract(&uniform_pixels(1200, [146, 201, 19], 201.0));
    assert_eq!(colored.slab_pos, -1);
}
