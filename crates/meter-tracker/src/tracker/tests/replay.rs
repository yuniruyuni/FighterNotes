use frame_meter::{CellState, RowObs};

use super::{insert_read, shared, MeterTracker};
use crate::tracker::WinEntry;

#[test]
fn diverge_step_counts_only_consecutive_wrapped_edges() {
    let mut tracker = MeterTracker::new();
    assert!(!tracker.diverge_step(5));
    assert_eq!((tracker.divergence, tracker.divergent_edge), (1, Some(5)));
    assert!(!tracker.diverge_step(5));
    assert_eq!(tracker.divergence, 1);
    assert!(!tracker.diverge_step(6));
    assert_eq!(tracker.divergence, 2);
    assert!(tracker.diverge_step(7));
    assert_eq!(tracker.divergence, 3);

    tracker.divergence = 1;
    tracker.divergent_edge = Some(79);
    assert!(!tracker.diverge_step(0));
    assert_eq!(tracker.divergence, 2);
}

#[test]
fn reset_replay_rewinds_old_data_and_replays_divergent_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(5);
    tracker.absolute_frame = Some(7);
    tracker.divergence = 2;
    for absolute in 5..=7 {
        insert_read(&mut tracker, "left", absolute, "active", 1.0, false);
        insert_read(&mut tracker, "right", absolute, "stun", 1.0, false);
        tracker.dwell.insert(absolute, [absolute, 20]);
    }
    tracker.dwell.get_mut(&6).unwrap()[1] = 11;
    tracker.video_map.insert(10, (0, 5));
    tracker.video_map.insert(11, (0, 6));
    tracker.video_map.insert(12, (0, 7));

    let mut observation = RowObs::empty();
    observation.states.fill(CellState::Counter);
    observation.rescued.fill(true);
    tracker.window = vec![
        WinEntry {
            vf: 10,
            left: shared(observation.clone()),
            right: shared(observation.clone()),
            vote_ok: true,
            prev_abs: Some(5),
        },
        WinEntry {
            vf: 11,
            left: shared(observation.clone()),
            right: shared(observation.clone()),
            vote_ok: true,
            prev_abs: Some(6),
        },
        WinEntry {
            vf: 12,
            left: shared(observation.clone()),
            right: shared(observation),
            vote_ok: true,
            prev_abs: Some(7),
        },
    ];

    tracker.reset_replay(20);

    assert_eq!(tracker.segment_id, 1);
    assert_eq!(tracker.absolute_frame, Some(20));
    assert_eq!(tracker.video_map[&10], (0, 5));
    assert_eq!(tracker.video_map[&11], (1, 19));
    assert_eq!(tracker.video_map[&12], (1, 20));
    let old_entries = &tracker.left.segments[0].entries;
    assert_eq!(
        old_entries
            .iter()
            .map(|entry| entry.game_frame)
            .collect::<Vec<_>>(),
        [5, 6]
    );
    assert_eq!(old_entries[1].video_frame_last, 10);
    assert_eq!(tracker.reads["left"][&19].0, "counter");
    assert_eq!(tracker.reads["left"][&20].0, "counter");
    assert_eq!(
        tracker.right.segments[0]
            .entries
            .iter()
            .map(|entry| (entry.game_frame, entry.state.as_str()))
            .collect::<Vec<_>>(),
        [(5, "stun"), (6, "stun")]
    );
}
