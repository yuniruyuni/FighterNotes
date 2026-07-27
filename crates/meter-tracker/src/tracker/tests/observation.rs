use std::collections::HashSet;

use frame_meter::{BrightClass, CellState, RowObs};

use crate::calibration::{BLACKISH_V_MAX, DIFF_V_MIN, DIFF_WF_MIN};

use super::{tracker_at, MeterTracker};

#[test]
fn changed_cells_requires_visual_and_classification_changes() {
    let mut tracker = MeterTracker::new();
    let mut previous_left = RowObs::empty();
    let previous_right = RowObs::empty();
    previous_left.v[2] = 100.0;
    previous_left.v[3] = 100.0;
    previous_left.v[9] = 100.0;
    previous_left.wf[4] = 0.0;
    previous_left.wf[5] = 0.0;
    previous_left.wf[10] = 0.5;
    tracker.previous = Some((previous_left.clone(), previous_right.clone()));

    let mut left = previous_left;
    left.v[2] += DIFF_V_MIN;
    left.states[2] = CellState::Counter;
    left.v[3] += DIFF_V_MIN + 1.0;
    left.states[3] = CellState::Counter;
    left.wf[4] = DIFF_WF_MIN;
    left.states[4] = CellState::Counter;
    left.wf[5] = DIFF_WF_MIN + 0.01;
    left.bright[5] = BrightClass::Fresh;
    left.v[6] = DIFF_V_MIN + 1.0;
    left.states[7] = CellState::Active;
    left.v[9] += 5.0;
    left.states[9] = CellState::Counter;
    left.wf[10] += 0.05;
    left.states[10] = CellState::Counter;

    let mut right = previous_right;
    right.v[8] = DIFF_V_MIN + 1.0;
    right.states[8] = CellState::Active;

    assert_eq!(
        tracker.changed_cells(&left, &right),
        HashSet::from([3, 5, 8])
    );
}

#[test]
fn changed_cells_and_wipe_count_are_neutral_without_previous_frame() {
    let tracker = MeterTracker::new();
    assert!(tracker
        .changed_cells(&RowObs::empty(), &RowObs::empty())
        .is_empty());
    assert_eq!(tracker.wipe_count(&RowObs::empty(), &RowObs::empty()), 0);
}

#[test]
fn wipe_count_uses_strict_brightness_boundaries_and_larger_side() {
    let mut tracker = MeterTracker::new();
    let mut previous_left = RowObs::empty();
    let mut previous_right = RowObs::empty();
    let mut left = RowObs::empty();
    let mut right = RowObs::empty();

    previous_left.v[..4].copy_from_slice(&[70.0, 71.0, 71.0, 100.0]);
    left.v[..4].copy_from_slice(&[24.0, 25.0, 24.0, 0.0]);
    previous_right.v[..3].fill(100.0);
    right.v[..3].fill(0.0);
    tracker.previous = Some((previous_left, previous_right));

    assert_eq!(tracker.wipe_count(&left, &right), 3);
}

#[test]
fn wipe_count_excludes_each_exact_boundary() {
    let mut tracker = MeterTracker::new();
    let mut previous = RowObs::empty();
    let mut current = RowObs::empty();
    previous.v[0] = 70.0;
    current.v[0] = 24.0;
    previous.v[1] = 71.0;
    current.v[1] = 25.0;
    tracker.previous = Some((previous, RowObs::empty()));

    assert_eq!(tracker.wipe_count(&current, &RowObs::empty()), 0);
}

#[test]
fn circular_delta_and_edge_selection_preserve_wraparound_ranking() {
    assert_eq!(MeterTracker::circ_delta(0, 79), 1);
    assert_eq!(MeterTracker::circ_delta(79, 0), -1);
    assert_eq!(MeterTracker::circ_delta(40, 0), 40);
    assert_eq!(MeterTracker::circ_delta(41, 0), -39);

    let tracker = MeterTracker::new();
    assert_eq!(tracker.select_edge(-1, -1), -1);
    assert_eq!(tracker.select_edge(5, 8), 8);

    let tracker = tracker_at(159);
    assert_eq!(tracker.select_edge(0, 1), 0);
    assert_eq!(tracker.select_edge(79, 2), 79);
    assert_eq!(tracker.select_edge(2, 78), 2);
    assert_eq!(tracker.select_edge(1, 2), 1);
    assert_eq!(tracker.select_edge(78, 77), 78);
}

#[test]
fn near_front_accepts_only_current_through_four_cells_ahead() {
    assert!(MeterTracker::near_front(&HashSet::from([0]), 79));
    assert!(MeterTracker::near_front(&HashSet::from([14]), 10));
    assert!(!MeterTracker::near_front(&HashSet::from([9]), 10));
    assert!(!MeterTracker::near_front(&HashSet::from([15]), 10));
}

#[test]
fn all_blackish_uses_a_strict_upper_bound() {
    let mut observation = RowObs::empty();
    observation.v.fill(BLACKISH_V_MAX - 1.0);
    assert!(MeterTracker::all_blackish(&observation));
    observation.v[40] = BLACKISH_V_MAX;
    assert!(!MeterTracker::all_blackish(&observation));
}
