use frame_meter::{CellState, RowObs};

use crate::calibration::{FREEZE_TIMEOUT, RESET_DIVERGENCE, WIPE_GUARD_MIN_CELLS};

use super::{lit_observation, MeterTracker};

#[test]
fn update_opens_only_matching_or_consecutive_edge_candidate() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(5), RowObs::empty());
    assert_eq!(tracker.open_candidate, Some(5));
    assert_eq!(tracker.absolute_frame, None);

    tracker.update(1, lit_observation(8), RowObs::empty());
    assert_eq!(tracker.open_candidate, Some(8));
    assert_eq!(tracker.absolute_frame, None);

    tracker.update(2, lit_observation(9), RowObs::empty());
    assert_eq!(tracker.open_candidate, None);
    assert_eq!(tracker.absolute_frame, Some(9));
    assert_eq!(tracker.video_map[&1], (0, 8));
    assert_eq!(tracker.video_map[&2], (0, 9));

    let mut equal = MeterTracker::new();
    equal.update(0, lit_observation(8), RowObs::empty());
    equal.update(1, lit_observation(8), RowObs::empty());
    assert_eq!(equal.absolute_frame, Some(8));
}

#[test]
fn update_clears_candidate_on_missing_edge_and_keeps_bounded_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_candidate = Some(5);
    for video_frame in 0..4 {
        tracker.update(video_frame, RowObs::empty(), RowObs::empty());
    }

    assert_eq!(tracker.open_candidate, None);
    assert_eq!(tracker.window.len(), RESET_DIVERGENCE as usize);
    assert_eq!(
        tracker
            .window
            .iter()
            .map(|entry| entry.vf)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
}

#[test]
fn update_rejects_vote_at_exact_wipe_guard_count() {
    let mut tracker = MeterTracker::new();
    let mut previous = lit_observation(-1);
    let mut current = previous.clone();
    previous.v[..WIPE_GUARD_MIN_CELLS as usize].fill(100.0);
    current.v[..WIPE_GUARD_MIN_CELLS as usize].fill(0.0);
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(0, current, lit_observation(-1));
    assert!(!tracker.window.last().unwrap().vote_ok);
}

#[test]
fn update_advances_one_and_resyncs_small_positive_delta() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(79);
    tracker.divergence = 2;
    tracker.divergent_edge = Some(30);
    tracker.previous = Some((lit_observation(79), lit_observation(-1)));
    tracker.update(1, lit_observation(0), lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(80));
    assert_eq!((tracker.divergence, tracker.divergent_edge), (0, None));
    assert_eq!(tracker.still_frames, 0);

    tracker.previous = Some((lit_observation(0), lit_observation(-1)));
    tracker.update(2, lit_observation(3), lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(83));
}

#[test]
fn update_uses_changed_front_during_unconfirmed_positive_divergence() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(10);
    let previous = lit_observation(10);
    let mut current = previous.clone();
    current.fresh_edge = 15;
    current.v[11] = 120.0;
    current.states[11] = CellState::Counter;
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));

    assert_eq!(tracker.absolute_frame, Some(11));
    assert_eq!((tracker.divergence, tracker.divergent_edge), (1, Some(15)));
}

#[test]
fn update_handles_negative_zero_and_missing_edges_with_vote() {
    for edge in [9, 10, -1] {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(10);
        tracker.divergence = 2;
        tracker.divergent_edge = Some(30);
        let previous = lit_observation(10);
        let mut current = previous.clone();
        current.fresh_edge = edge;
        current.v[11] = 120.0;
        current.states[11] = CellState::Counter;
        tracker.previous = Some((previous, lit_observation(-1)));

        tracker.update(1, current, lit_observation(-1));
        assert_eq!(tracker.absolute_frame, Some(11), "edge={edge}");
        if edge == 9 {
            assert_eq!((tracker.divergence, tracker.divergent_edge), (1, Some(9)));
        } else {
            assert_eq!((tracker.divergence, tracker.divergent_edge), (0, None));
        }
    }
}

#[test]
fn update_does_not_use_negative_edge_change_when_vote_is_rejected() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(10);
    let previous = lit_observation(10);
    let mut current = previous.clone();
    current.fresh_edge = 9;
    current.v[..WIPE_GUARD_MIN_CELLS as usize].fill(0.0);
    current.states[11] = CellState::Counter;
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(10));
}

#[test]
fn update_uses_wrapped_predicted_cell_for_large_positive_divergence() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(79);
    let previous = lit_observation(79);
    let mut current = previous.clone();
    current.fresh_edge = 5;
    current.v[4] = 120.0;
    current.states[4] = CellState::Counter;
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(80));
}

#[test]
fn update_requires_both_sides_black_when_edge_is_missing() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(10);
    let observation = lit_observation(-1);
    tracker.previous = Some((observation.clone(), RowObs::empty()));

    tracker.update(1, observation, RowObs::empty());
    assert_eq!(tracker.absolute_frame, Some(10));
}

#[test]
fn update_requires_vote_before_missing_edge_change_can_advance() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(30);
    let previous = lit_observation(-1);
    let mut current = previous.clone();
    current.v[..WIPE_GUARD_MIN_CELLS as usize].fill(0.0);
    current.states[31] = CellState::Counter;
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(30));
}

#[test]
fn update_passes_advanced_state_to_slab_resolution() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(10);
    let mut previous = lit_observation(10);
    previous.cols = Some((0..80).map(|cell| (cell + 1) as f32).collect());
    previous.cols_w = 1;
    let mut current = lit_observation(11);
    current.states[11] = CellState::Other;
    current.cols = Some((0..80).map(|cell| cell as f32).collect());
    current.cols_w = 1;
    tracker.previous = Some((previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));

    assert_eq!(tracker.reads["left"][&11].0, "empty");
}

#[test]
fn update_closes_on_black_frames_or_freeze_timeout() {
    let mut black = MeterTracker::new();
    black.open_segment(10);
    black.previous = Some((lit_observation(10), lit_observation(-1)));
    black.update(1, RowObs::empty(), RowObs::empty());
    assert_eq!(black.absolute_frame, None);

    let mut frozen = MeterTracker::new();
    frozen.open_segment(10);
    frozen.still_frames = FREEZE_TIMEOUT - 1;
    let observation = lit_observation(10);
    frozen.previous = Some((observation.clone(), lit_observation(-1)));
    frozen.update(1, observation, lit_observation(-1));
    assert_eq!(frozen.absolute_frame, None);
    assert_eq!(frozen.still_frames, FREEZE_TIMEOUT);
}
