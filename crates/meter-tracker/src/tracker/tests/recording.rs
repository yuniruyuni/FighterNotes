use frame_meter::{CellState, RowObs, CELL_COUNT};

use crate::calibration::{READ_DIM_CONF, READ_EARLY_CONF, READ_FADE_CONF, READ_FRESH_CONF};

use super::{insert_read, MeterTracker};

fn columns_with<F>(mut value: F) -> Vec<f32>
where
    F: FnMut(usize) -> f32,
{
    (0..CELL_COUNT).map(&mut value).collect()
}

#[test]
fn record_maps_wrapped_cell_and_assigns_read_confidences() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(159);
    let mut left = RowObs::empty();
    let mut right = RowObs::empty();
    left.states[79] = CellState::Active;
    right.states[79] = CellState::Stun;
    left.states[78] = CellState::Counter;
    left.states[77] = CellState::Parry;
    left.states[76] = CellState::PunishCounter;

    tracker.record(10, &left, &right, true, false);
    tracker.record(12, &left, &right, false, false);

    assert_eq!(tracker.video_map[&10], (0, 159));
    assert_eq!(tracker.dwell[&159], [10, 12]);
    assert_eq!(tracker.reads["left"][&159].0, "active");
    assert_eq!(tracker.reads["right"][&159].0, "stun");
    assert_eq!(tracker.reads["left"][&159].1, READ_FADE_CONF);
    assert_eq!(tracker.reads["left"][&158].1, READ_EARLY_CONF);
    assert_eq!(tracker.reads["left"][&157].1, READ_EARLY_CONF);
    assert_eq!(tracker.reads["left"][&156].1, READ_FRESH_CONF);
}

#[test]
fn advanced_record_resolves_other_or_rescued_state_with_matching_previous_side() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(5);
    let mut current_left = RowObs::empty();
    current_left.states[5] = CellState::Other;
    current_left.cols = Some(columns_with(|cell| cell as f32));
    current_left.cols_w = 1;
    let mut previous_left = RowObs::empty();
    previous_left.cols = Some(columns_with(|cell| (cell + 1) as f32));
    previous_left.cols_w = 1;
    let mut previous_right = RowObs::empty();
    previous_right.cols = Some(columns_with(|cell| (cell * 40 + 20) as f32));
    previous_right.cols_w = 1;

    let mut current_right = RowObs::empty();
    current_right.states[5] = CellState::Counter;
    current_right.rescued[5] = true;
    current_right.cols = Some(columns_with(|cell| (cell * 40) as f32));
    current_right.cols_w = 1;
    tracker.previous = Some((previous_left, previous_right));

    tracker.record(10, &current_left, &current_right, true, true);

    assert_eq!(tracker.reads["left"][&5].0, "empty");
    assert_eq!(tracker.reads["right"][&5].0, "other");
}

#[test]
fn record_reads_previous_lap_dim_cells_only_within_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(80);
    let mut observation = RowObs::empty();
    observation.states[0] = CellState::Active;
    observation.states[78] = CellState::Counter;
    observation.states[79] = CellState::Stun;

    tracker.record(10, &observation, &observation, true, false);

    assert_eq!(tracker.reads["left"][&80].0, "active");
    assert_eq!(tracker.reads["left"][&78].1, READ_DIM_CONF);
    assert_eq!(tracker.reads["left"][&79].1, READ_DIM_CONF);
    assert!(!tracker.reads["left"].contains_key(&77));
}

#[test]
fn record_reads_dim_cells_at_later_lap_and_exact_window_boundary() {
    for absolute in [90, 160] {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(absolute);
        let mut observation = RowObs::empty();
        observation.states[78] = CellState::Counter;
        observation.states[79] = CellState::Stun;

        tracker.record(10, &observation, &observation, true, false);

        let previous_lap = absolute / CELL_COUNT as i64 - 1;
        assert_eq!(
            tracker.reads["left"][&(previous_lap * CELL_COUNT as i64 + 78)].1,
            READ_DIM_CONF
        );
        assert_eq!(
            tracker.reads["left"][&(previous_lap * CELL_COUNT as i64 + 79)].1,
            READ_DIM_CONF
        );
    }
}

#[test]
fn record_finalizes_entries_older_than_read_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(20);
    insert_read(&mut tracker, "left", 5, "stun", 1.0, false);
    tracker.dwell.insert(5, [2, 3]);

    tracker.record(20, &RowObs::empty(), &RowObs::empty(), false, false);

    assert!(!tracker.reads["left"].contains_key(&5));
    assert_eq!(tracker.left.segments[0].entries[0].game_frame, 5);
}
