use frame_meter::{CellState, RowObs, CELL_COUNT};

use crate::tracker::reading::slab::{compare_columns, mean_absolute_difference};

use super::{insert_read, tracker_at};

fn columns_with<F>(width: usize, mut value: F) -> Vec<f32>
where
    F: FnMut(usize, usize) -> f32,
{
    let mut columns = vec![0.0; CELL_COUNT * width];
    for cell in 0..CELL_COUNT {
        for index in 0..width {
            columns[cell * width + index] = value(cell, index);
        }
    }
    columns
}

fn observation_with_columns(columns: Vec<f32>, width: usize) -> RowObs {
    let mut observation = RowObs::empty();
    observation.cols = Some(columns);
    observation.cols_w = width;
    observation
}

#[test]
fn mean_absolute_difference_uses_all_pairs_and_left_length() {
    assert_eq!(mean_absolute_difference(&[], &[]), 0.0);
    assert_eq!(mean_absolute_difference(&[1.0, 4.0], &[3.0, 8.0]), 3.0);
}

#[test]
fn compare_columns_selects_requested_wrapped_cells_and_common_width() {
    let current = columns_with(2, |cell, index| (cell * 100 + index) as f32);
    let previous = columns_with(3, |cell, index| (cell * 100 + index + 3) as f32);
    assert_eq!(
        compare_columns(&current, 2, &previous, 3, 0, &[0, 1], &[0, 1]),
        3.0
    );
    assert_eq!(
        compare_columns(&current, 0, &previous, 3, 5, &[0], &[0]),
        f64::MAX
    );
    assert_eq!(
        compare_columns(&current, 2, &previous, 0, 5, &[0], &[0]),
        f64::MAX
    );
}

#[test]
fn resolve_slab_detects_sliding_and_static_columns() {
    let tracker = tracker_at(20);
    let sliding_current = columns_with(1, |cell, _| cell as f32);
    let sliding_previous = columns_with(1, |cell, _| (cell + 1) as f32);
    let current = observation_with_columns(sliding_current, 1);
    let previous = observation_with_columns(sliding_previous, 1);
    assert_eq!(
        tracker.resolve_slab(&current, Some(&previous), 20, "left"),
        "empty"
    );

    let static_columns = columns_with(1, |cell, _| (cell * 20) as f32);
    let current = observation_with_columns(static_columns.clone(), 1);
    let previous = observation_with_columns(static_columns, 1);
    assert_eq!(
        tracker.resolve_slab(&current, Some(&previous), 20, "left"),
        "empty"
    );
}

#[test]
fn resolve_slab_falls_back_to_run_then_previous_cell_state() {
    let current_columns = columns_with(1, |cell, _| (cell * 40) as f32);
    let previous_columns = columns_with(1, |cell, _| (cell * 40 + 20) as f32);
    let mut current = observation_with_columns(current_columns, 1);
    let previous = observation_with_columns(previous_columns, 1);

    let mut tracker = tracker_at(20);
    insert_read(&mut tracker, "left", 19, "stun", 1.0, false);
    assert_eq!(
        tracker.resolve_slab(&current, Some(&previous), 20, "left"),
        "stun"
    );

    tracker.reads.get_mut("left").unwrap().clear();
    current.states[19] = CellState::Active;
    assert_eq!(
        tracker.resolve_slab(&current, Some(&previous), 20, "left"),
        "active"
    );
    current.states[19] = CellState::Empty;
    assert_eq!(
        tracker.resolve_slab(&current, Some(&previous), 20, "left"),
        "other"
    );
    assert_eq!(tracker.resolve_slab(&current, None, 20, "left"), "other");
}
