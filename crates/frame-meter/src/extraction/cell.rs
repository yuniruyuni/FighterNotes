use crate::classification::classify_cell_pair;
use crate::color::{Bgr, QuantizedModeScratch};
use crate::constants::{BLACKISH_PATCH_V, HIGHLIGHT_V_MIN};
use crate::model::{BrightClass, CellState};
use crate::palette::state_quality;
use crate::rescue::dominant_color_family;

use super::metrics::CellBounds;
use super::source::RowPixels;

pub(crate) struct ClassifiedCell {
    pub(crate) bgr: Bgr,
    pub(crate) state: CellState,
    pub(crate) bright: BrightClass,
    pub(crate) rescued: bool,
    pub(crate) quality: f32,
}

pub(crate) fn classify(
    pixels: &RowPixels,
    bounds: CellBounds,
    mean_value: f32,
    region1: &mut Vec<[u8; 3]>,
    region2: &mut Vec<[u8; 3]>,
    color_scratch: &mut QuantizedModeScratch,
) -> ClassifiedCell {
    collect_region(region1, pixels, &pixels.region1_rows, bounds);
    collect_region(region2, pixels, &pixels.region2_rows, bounds);
    let mut a_bgr = color_scratch.mean(region1);
    let mut b_bgr = color_scratch.mean(region2);
    let (mut state, mut bright) = classify_cell_pair(a_bgr, b_bgr);
    if state == CellState::Empty && mean_value >= HIGHLIGHT_V_MIN {
        state = CellState::Other;
    }

    let mut rescued = false;
    if (state == CellState::Empty || state == CellState::Other) && mean_value > BLACKISH_PATCH_V {
        let first_family = dominant_color_family(region1);
        let second_family = dominant_color_family(region2);
        if let (Some((family1, mean1)), Some((family2, mean2))) = (first_family, second_family) {
            if family1 == family2 {
                let (rescued_state, rescued_bright) = classify_cell_pair(mean1, mean2);
                a_bgr = mean1;
                b_bgr = mean2;
                state = rescued_state;
                bright = rescued_bright;
                rescued = true;
            }
        }
    }

    let quality = state_quality(&state, a_bgr, b_bgr);
    ClassifiedCell {
        bgr: b_bgr,
        state,
        bright,
        rescued,
        quality,
    }
}

fn collect_region(
    output: &mut Vec<[u8; 3]>,
    pixels: &RowPixels,
    region_rows: &[usize],
    bounds: CellBounds,
) {
    output.clear();
    for &local_row in region_rows {
        let row = pixels.trim_y + local_row;
        if let Some(row_pixels) = pixels.bgr.chunks_exact(pixels.width).nth(row) {
            output.extend_from_slice(&row_pixels[bounds.x1..bounds.x2]);
        }
    }
}
