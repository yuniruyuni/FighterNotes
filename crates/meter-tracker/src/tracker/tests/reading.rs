use frame_meter::{CellState, RowObs, CELL_COUNT};

use crate::calibration::LABEL_DIGIT_MIN;
use crate::tracker::reading::{is_information, is_read_information};

use super::{digit_correlations, insert_read, observation_with_state, tracker_at, MeterTracker};

#[test]
fn information_predicates_distinguish_unknown_read_state() {
    for state in ["counter", "stun", "inv_full"] {
        assert!(is_information(state));
        assert!(is_read_information(state));
    }
    for state in ["empty", "other"] {
        assert!(!is_information(state));
        assert!(!is_read_information(state));
    }
    assert!(!is_information("unknown"));
    assert!(is_read_information("unknown"));
}

#[test]
fn run_state_finds_nearest_information_within_three_frames() {
    let mut tracker = tracker_at(10);
    insert_read(&mut tracker, "left", 9, "empty", 1.0, false);
    insert_read(&mut tracker, "left", 8, "active", 1.0, false);
    insert_read(&mut tracker, "left", 7, "stun", 1.0, false);
    assert_eq!(tracker.run_state("left").as_deref(), Some("active"));

    tracker.reads.get_mut("left").unwrap().remove(&8);
    assert_eq!(tracker.run_state("left").as_deref(), Some("stun"));
    tracker.absolute_frame = None;
    assert_eq!(tracker.run_state("left"), None);
}

#[test]
fn run_back_len_combines_reads_and_emitted_until_state_changes() {
    let mut tracker = tracker_at(5);
    insert_read(&mut tracker, "left", 4, "active", 1.0, false);
    tracker
        .emitted
        .get_mut("left")
        .unwrap()
        .insert(3, "active".to_string());
    insert_read(&mut tracker, "left", 2, "active", 1.0, false);
    insert_read(&mut tracker, "left", 1, "counter", 1.0, false);

    assert_eq!(
        tracker.run_back_len("left"),
        (Some("active".to_string()), 3)
    );
    tracker
        .reads
        .get_mut("left")
        .unwrap()
        .insert(4, ("empty".to_string(), 1.0, false));
    assert_eq!(tracker.run_back_len("left"), (None, 0));
    tracker.absolute_frame = Some(0);
    assert_eq!(tracker.run_back_len("left"), (None, 0));
}

#[test]
fn run_back_len_stops_at_a_missing_frame() {
    let mut tracker = tracker_at(5);
    insert_read(&mut tracker, "left", 4, "active", 1.0, false);
    insert_read(&mut tracker, "left", 2, "active", 1.0, false);

    assert_eq!(
        tracker.run_back_len("left"),
        (Some("active".to_string()), 1)
    );
}

#[test]
fn run_back_len_does_not_resume_after_a_different_state() {
    let mut tracker = tracker_at(6);
    insert_read(&mut tracker, "left", 5, "active", 1.0, false);
    insert_read(&mut tracker, "left", 4, "counter", 1.0, false);
    insert_read(&mut tracker, "left", 3, "active", 1.0, false);

    assert_eq!(
        tracker.run_back_len("left"),
        (Some("active".to_string()), 1)
    );
}

#[test]
fn digit_covered_handles_missing_wrapped_and_threshold_scores() {
    let mut observation = RowObs::empty();
    assert!(!MeterTracker::digit_covered(&observation, 0));

    let mut correlations = digit_correlations();
    correlations[CELL_COUNT - 1][4] = LABEL_DIGIT_MIN as f32;
    observation.digit_corr = Some(correlations);
    assert!(MeterTracker::digit_covered(&observation, -1));

    observation.digit_corr.as_mut().unwrap()[CELL_COUNT - 1][4] = LABEL_DIGIT_MIN as f32 - 0.01;
    assert!(!MeterTracker::digit_covered(&observation, -1));
    observation.digit_corr = Some(vec![[-1.0; 10]; 1]);
    assert!(!MeterTracker::digit_covered(&observation, -1));
}

#[test]
fn better_read_preserves_information_coverage_and_strict_confidence_order() {
    assert!(MeterTracker::better_read(None, "active", 0.1, false));

    let informational = ("active".to_string(), 0.8, false);
    assert!(MeterTracker::better_read(
        Some(&informational),
        "stun",
        0.9,
        false
    ));
    assert!(!MeterTracker::better_read(
        Some(&informational),
        "stun",
        0.8,
        false
    ));
    assert!(!MeterTracker::better_read(
        Some(&informational),
        "stun",
        1.0,
        true
    ));
    assert!(!MeterTracker::better_read(
        Some(&informational),
        "empty",
        1.0,
        false
    ));

    let covered = ("active".to_string(), 0.8, true);
    assert!(MeterTracker::better_read(Some(&covered), "stun", 0.9, true));
    let empty = ("empty".to_string(), 0.8, false);
    assert!(MeterTracker::better_read(
        Some(&empty),
        "active",
        0.1,
        false
    ));
    assert!(MeterTracker::better_read(Some(&empty), "other", 0.9, false));
    assert!(!MeterTracker::better_read(
        Some(&empty),
        "other",
        0.8,
        false
    ));
}

#[test]
fn dim_read_returns_only_information_states() {
    let mut observation = observation_with_state(CellState::Empty);
    observation.states[3] = CellState::Counter;
    observation.states[4] = CellState::Other;
    observation.states[5] = CellState::Unknown;
    assert_eq!(
        MeterTracker::dim_read(&observation, 3).as_deref(),
        Some("counter")
    );
    assert_eq!(MeterTracker::dim_read(&observation, 4), None);
    assert_eq!(MeterTracker::dim_read(&observation, 5), None);
}
