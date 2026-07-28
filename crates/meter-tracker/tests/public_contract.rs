use frame_meter::RowObs;
use meter_tracker::{MeterTimeline, MeterTracker, TimelineEntry, TimelineSegment};

#[test]
fn crate_root_keeps_the_tracker_api() {
    fn assert_send<T: Send>() {}

    assert_send::<MeterTracker>();
    let _: fn() -> MeterTracker = MeterTracker::new;
    let _: fn(&mut MeterTracker, i64, RowObs, RowObs) = MeterTracker::update;
    let _: fn(&mut MeterTracker) = MeterTracker::finish;
    let _: fn(&mut MeterTracker) = MeterTracker::close;
    let _: fn(&mut MeterTracker) = MeterTracker::suspend;
    let _: fn(&MeterTracker) -> Option<(usize, usize)> = MeterTracker::digit_window_hint;
    let _: fn(&MeterTimeline, u32) -> Vec<bool> = MeterTimeline::stun_per_frame;

    let mut tracker = MeterTracker::default();
    tracker.suspend();
    tracker.close();
}

#[test]
fn timeline_projects_stun_entries_to_video_frames() {
    let timeline = MeterTimeline {
        side: "left".to_string(),
        segments: vec![TimelineSegment {
            segment_id: 3,
            entries: vec![
                TimelineEntry {
                    game_frame: 8,
                    state: "stun".to_string(),
                    video_frame_first: 1,
                    video_frame_last: 3,
                    confidence: 1.0,
                },
                TimelineEntry {
                    game_frame: 9,
                    state: "active".to_string(),
                    video_frame_first: 4,
                    video_frame_last: 5,
                    confidence: 0.9,
                },
            ],
        }],
    };

    assert_eq!(
        timeline.stun_per_frame(6),
        [false, true, true, true, false, false],
    );
}
