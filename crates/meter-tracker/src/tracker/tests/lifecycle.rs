use frame_meter::RowObs;

use super::{insert_read, MeterTracker};

#[test]
fn open_and_close_segment_reset_state_and_finalize_pending_reads() {
    let mut tracker = MeterTracker::new();
    insert_read(&mut tracker, "left", 1, "stale", 1.0, false);
    tracker.dwell.insert(1, [1, 2]);
    tracker.divergence = 4;
    tracker.divergent_edge = Some(7);
    tracker.still_frames = 9;

    tracker.open_segment(4);
    assert_eq!(tracker.segment_id, 0);
    assert_eq!(tracker.absolute_frame, Some(4));
    assert!(tracker.reads["left"].is_empty());
    assert!(tracker.dwell.is_empty());
    assert_eq!(tracker.divergence, 0);
    assert_eq!(tracker.divergent_edge, None);
    assert_eq!(tracker.still_frames, 0);

    insert_read(&mut tracker, "left", 4, "active", 0.8765, false);
    insert_read(&mut tracker, "right", 4, "stun", 1.0, false);
    tracker.dwell.insert(4, [10, 12]);
    tracker.previous = Some((RowObs::empty(), RowObs::empty()));
    tracker.close();

    assert_eq!(tracker.absolute_frame, None);
    assert!(tracker.previous.is_none());
    let left = &tracker.left.segments[0].entries[0];
    assert_eq!(
        (
            left.game_frame,
            left.state.as_str(),
            left.video_frame_first,
            left.video_frame_last,
            left.confidence,
        ),
        (4, "active", 10, 12, 0.877)
    );
}

#[test]
fn suspend_forgets_only_previous_observation() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(3);
    tracker.previous = Some((RowObs::empty(), RowObs::empty()));
    tracker.suspend();

    assert!(tracker.previous.is_none());
    assert_eq!(tracker.absolute_frame, Some(3));
}

#[test]
fn emission_uses_unknown_state_and_missing_video_sentinels() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(2);
    insert_read(&mut tracker, "left", 2, "counter", 0.1234, false);
    tracker.close_segment();

    let left = &tracker.left.segments[0].entries[0];
    let right = &tracker.right.segments[0].entries[0];
    assert_eq!(
        (
            left.state.as_str(),
            left.video_frame_first,
            left.video_frame_last,
            left.confidence,
        ),
        ("counter", -1, -1, 0.123)
    );
    assert_eq!(
        (
            right.state.as_str(),
            right.video_frame_first,
            right.video_frame_last,
            right.confidence,
        ),
        ("unknown", -1, -1, 0.0)
    );
}
