use crate::classification::fresh_v_min_for;
use crate::constants::{CELL_COUNT, EMPTY_V_MAX};
use crate::model::{BrightClass, CellState};

fn has_front_gap(v: &[f32], index: usize) -> bool {
    let next: Vec<f32> = [index + 1, index + 2]
        .iter()
        .filter(|&&candidate| candidate < CELL_COUNT)
        .map(|&candidate| v[candidate])
        .collect();
    if next.is_empty() {
        return true;
    }
    let min_next = next.iter().copied().fold(f32::MAX, f32::min);
    let threshold = EMPTY_V_MAX.max(0.5 * v[index]);
    min_next < threshold
}

/// Returns the rightmost fresh colored-cell index, or -1 when none exists.
pub fn fresh_color_edge(
    v: &[f32],
    _wf: &[f32],
    states: &[CellState],
    bright: &[BrightClass],
) -> i32 {
    for index in (0..CELL_COUNT).rev() {
        let state = &states[index];
        let colored = state.is_stripe() || fresh_v_min_for(state).is_some();
        let is_fresh = colored
            && (bright[index] == BrightClass::Fresh
                || (bright[index] == BrightClass::Low
                    && index > 0
                    && bright[index - 1] == BrightClass::Fresh));
        if is_fresh && has_front_gap(v, index) {
            return index as i32;
        }
    }
    -1
}
