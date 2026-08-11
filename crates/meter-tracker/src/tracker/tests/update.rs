use frame_meter::{CellState, RowObs};
use std::sync::Arc;

use crate::calibration::{FREEZE_TIMEOUT, RESET_DIVERGENCE, WIPE_GUARD_MIN_CELLS};

use super::{lit_observation, shared_pair, MeterTracker};

#[test]
fn digit_window_hint_tracks_the_wrapped_current_cell() {
    let mut tracker = MeterTracker::new();
    assert_eq!(tracker.digit_window_hint(), None);

    tracker.open_segment(79);
    assert_eq!(tracker.digit_window_hint(), Some((79, 12)));
    tracker.absolute_frame = Some(80);
    assert_eq!(tracker.digit_window_hint(), Some((0, 12)));
}

#[test]
fn update_shares_observations_between_window_and_previous_frame() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(-1), lit_observation(-1));

    let window = tracker.window.last().unwrap();
    let previous = tracker.previous.as_ref().unwrap();
    assert!(Arc::ptr_eq(&window.left, &previous.0));
    assert!(Arc::ptr_eq(&window.right, &previous.1));
}

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
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

    tracker.update(0, current, lit_observation(-1));
    assert!(!tracker.window.last().unwrap().vote_ok);
}

#[test]
fn update_advances_one_and_resyncs_small_positive_delta() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(79);
    tracker.divergence = 2;
    tracker.divergent_edge = Some(30);
    tracker.previous = Some(shared_pair(lit_observation(79), lit_observation(-1)));
    tracker.update(1, lit_observation(0), lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(80));
    assert_eq!(tracker.previous.as_ref().unwrap().0.fresh_edge, 0);
    assert_eq!((tracker.divergence, tracker.divergent_edge), (0, None));
    assert_eq!(tracker.still_frames, 0);

    tracker.previous = Some(shared_pair(lit_observation(0), lit_observation(-1)));
    tracker.update(2, lit_observation(3), lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(83));
}

#[test]
fn confirmed_large_divergence_resets_for_both_directions() {
    for (edge, previous_edge) in [(15, 14), (5, 4)] {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(10);
        tracker.divergence = RESET_DIVERGENCE - 1;
        tracker.divergent_edge = Some(previous_edge);
        tracker.previous = Some(shared_pair(lit_observation(10), RowObs::empty()));

        tracker.update(1, lit_observation(edge), RowObs::empty());

        assert_eq!(tracker.segment_id, 1, "edge={edge}");
        assert_eq!(tracker.absolute_frame, Some(edge as i64), "edge={edge}");
    }
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
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

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
        tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

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
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

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
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));
    assert_eq!(tracker.absolute_frame, Some(80));
}

#[test]
fn update_requires_both_sides_black_when_edge_is_missing() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(10);
    let observation = lit_observation(-1);
    tracker.previous = Some(shared_pair(observation.clone(), RowObs::empty()));

    tracker.update(1, observation, RowObs::empty());
    assert_eq!(tracker.absolute_frame, Some(10));
}

#[test]
fn update_requires_both_sides_black_when_an_edge_is_present() {
    let mut closed = MeterTracker::new();
    closed.open_segment(10);
    closed.previous = Some(shared_pair(lit_observation(10), lit_observation(10)));
    let mut black = RowObs::empty();
    black.fresh_edge = 10;
    closed.update(1, black.clone(), black.clone());
    assert_eq!(closed.absolute_frame, None);

    let mut one_black = MeterTracker::new();
    one_black.open_segment(10);
    one_black.previous = Some(shared_pair(lit_observation(10), lit_observation(10)));
    one_black.update(1, black, lit_observation(10));
    assert_eq!(one_black.absolute_frame, Some(10));
}

#[test]
fn update_requires_vote_before_missing_edge_change_can_advance() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(30);
    let previous = lit_observation(-1);
    let mut current = previous.clone();
    current.v[..WIPE_GUARD_MIN_CELLS as usize].fill(0.0);
    current.states[31] = CellState::Counter;
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

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
    tracker.previous = Some(shared_pair(previous, lit_observation(-1)));

    tracker.update(1, current, lit_observation(-1));

    assert_eq!(tracker.reads["left"][&11].0, "empty");
}

#[test]
fn update_closes_on_black_frames_or_freeze_timeout() {
    let mut black = MeterTracker::new();
    black.open_segment(10);
    black.previous = Some(shared_pair(lit_observation(10), lit_observation(-1)));
    black.update(1, RowObs::empty(), RowObs::empty());
    assert_eq!(black.absolute_frame, None);

    let mut frozen = MeterTracker::new();
    frozen.open_segment(10);
    frozen.still_frames = FREEZE_TIMEOUT - 1;
    let observation = lit_observation(10);
    frozen.previous = Some(shared_pair(observation.clone(), lit_observation(-1)));
    frozen.update(1, observation, lit_observation(-1));
    assert_eq!(frozen.absolute_frame, None);
    assert_eq!(frozen.still_frames, FREEZE_TIMEOUT);
}

/// 周回の先頭のセルから始まるメーターも読み始められる。境目を
/// 取り違えると、ラウンドの頭の 1 周分がまるごと落ちる。
#[test]
fn a_segment_can_open_at_the_very_first_cell() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(0), RowObs::empty());
    tracker.update(1, lit_observation(0), RowObs::empty());

    assert_eq!(tracker.absolute_frame, Some(0), "先頭のセルで開けていない");
}

/// 開いた瞬間には、その 1 つ前のフレームも同じ区間として記録する。
/// メーターは既にそこから動いている。
#[test]
fn opening_a_segment_also_records_the_frame_before_it() {
    let mut tracker = MeterTracker::new();
    tracker.update(7, lit_observation(8), RowObs::empty());
    tracker.update(8, lit_observation(8), RowObs::empty());

    assert_eq!(tracker.video_map[&7], (0, 8), "直前のフレームを捨てている");
    assert_eq!(tracker.video_map[&8], (0, 8));
}

#[test]
fn opening_preserves_stationary_other_reads() {
    let mut tracker = MeterTracker::new();
    let mut observation = lit_observation(8);
    observation.states[8] = CellState::Other;
    observation.cols = Some((0..80).map(|cell| cell as f32).collect());
    observation.cols_w = 1;

    tracker.update(7, observation.clone(), observation.clone());
    tracker.update(8, observation.clone(), observation);

    assert_eq!(tracker.reads["left"][&8].0, "other");
}

/// 直前のフレームの票が割れていたなら、そのフレームの色は採らない。
/// 開き直したからといって、読めていなかったものが読めた扱いにならない。
#[test]
fn the_recorded_previous_frame_keeps_its_own_vote() {
    let mut tracker = MeterTracker::new();
    let mut lit = lit_observation(8);
    lit.states[8] = CellState::Active;
    // 直前のフレームで大量のセルが黒へ落ち、票が割れる。
    let mut wiped = lit_observation(8);
    wiped.states[8] = CellState::Active;
    for value in wiped.v.iter_mut().take(WIPE_GUARD_MIN_CELLS as usize) {
        *value = 0.0;
    }
    tracker.previous = Some(shared_pair(lit_observation(-1), lit_observation(-1)));

    tracker.update(7, wiped.clone(), wiped);
    tracker.update(8, lit.clone(), lit);

    assert_eq!(tracker.video_map[&7], (0, 8), "時刻の対応まで捨てている");
    assert_eq!(
        tracker.dwell[&8],
        [7, 8],
        "直前のフレームを区間に含めていない"
    );
}

#[test]
fn a_rejected_previous_vote_cannot_override_the_opening_frame_colour() {
    let mut tracker = MeterTracker::new();
    tracker.previous = Some(shared_pair(lit_observation(-1), lit_observation(-1)));
    let mut rejected = lit_observation(8);
    rejected.states[8] = CellState::Counter;
    rejected.cols = Some((0..80).map(|cell| cell as f32).collect());
    rejected.cols_w = 1;
    rejected.v[..WIPE_GUARD_MIN_CELLS as usize].fill(0.0);
    let mut accepted = lit_observation(8);
    accepted.states[8] = CellState::Other;
    accepted.cols = Some((0..80).map(|cell| cell as f32).collect());
    accepted.cols_w = 1;

    tracker.update(7, rejected.clone(), rejected);
    tracker.update(8, accepted.clone(), accepted);

    assert_eq!(tracker.reads["left"][&8].0, "other");
}

/// 画面が真っ黒になれば、そこで区間は終わり。演出やリプレイの
/// 切り替わりでメーター自体が消える。
#[test]
fn an_all_black_frame_closes_the_segment() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(8), lit_observation(8));
    tracker.update(1, lit_observation(9), lit_observation(9));
    assert!(tracker.absolute_frame.is_some());

    tracker.update(2, RowObs::empty(), RowObs::empty());

    assert_eq!(tracker.absolute_frame, None, "真っ黒でも続けている");
}

/// 片側だけ黒いのは、演出でその側が隠れているだけ。区間は続く。
#[test]
fn one_black_row_does_not_close_the_segment() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(8), lit_observation(8));
    tracker.update(1, lit_observation(9), lit_observation(9));

    tracker.update(2, RowObs::empty(), lit_observation(10));

    assert!(tracker.absolute_frame.is_some(), "片側の黒で閉じている");
}

/// メーターが止まったまま長く続けば、そこで区間を切る。ポーズや
/// リプレイの停止でメーターが固まる。
#[test]
fn a_meter_frozen_for_too_long_closes_the_segment() {
    let mut tracker = MeterTracker::new();
    tracker.update(0, lit_observation(8), lit_observation(8));
    tracker.update(1, lit_observation(9), lit_observation(9));

    let mut still = lit_observation(9);
    still.states.fill(CellState::Active);
    for video_frame in 2..FREEZE_TIMEOUT + 1 {
        tracker.update(video_frame, still.clone(), still.clone());
        assert!(
            tracker.absolute_frame.is_some(),
            "{video_frame} で早く閉じている"
        );
    }

    tracker.update(FREEZE_TIMEOUT + 1, still.clone(), still);

    assert_eq!(tracker.absolute_frame, None, "止まったまま続けている");
}
