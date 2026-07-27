//! SF6 frame-meter detection from browser-order RGBA image buffers.

mod classification;
mod color;
mod constants;
mod digits;
mod edge;
mod extraction;
mod model;
mod palette;
mod rescue;

#[cfg(test)]
mod tests;

pub use classification::{brightness_class, classify_cell_pair, classify_cell_raw};
pub use constants::{
    BLACKISH_PATCH_V, CELL_COUNT, DIGIT_CHARS, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W, DIM_V_SCALE,
    EMPTY_V_MAX, FAMILY_ASSIGN_DIST, HIGHLIGHT_V_MIN, LIT_ROW_V_MIN, METER_STRIP_H, METER_STRIP_Y,
    PAIR_REJECT_DIST, RESCUE_MIN_FRAC, STRIPE_MAX_ROW_XSTD, STRIPE_MIN_CONTRAST,
    STRIPE_MIN_TRANSITIONS, STRIPE_WF_MIN,
};
pub use edge::fresh_color_edge;
pub use extraction::{extract_row_obs, extract_row_obs_from_strip};
pub use model::{BrightClass, CellState, RowObs};
