use meter_tracker::{MeterTimeline, TimelineEntry, TimelineSegment};

pub fn meter_run(state: &str, frames: std::ops::RangeInclusive<i64>) -> Vec<TimelineEntry> {
    frames
        .map(|frame| TimelineEntry {
            game_frame: frame,
            state: state.to_string(),
            video_frame_first: frame,
            video_frame_last: frame,
            confidence: 1.0,
        })
        .collect()
}

pub fn meter_pause(state: &str, first: i64, last: i64) -> TimelineEntry {
    TimelineEntry {
        game_frame: first,
        state: state.to_string(),
        video_frame_first: first,
        video_frame_last: last,
        confidence: 1.0,
    }
}

pub fn timeline(side: &str, mut entries: Vec<TimelineEntry>) -> MeterTimeline {
    entries.sort_by_key(|entry| entry.video_frame_first);
    MeterTimeline {
        side: side.to_string(),
        segments: vec![TimelineSegment {
            segment_id: 0,
            entries,
        }],
    }
}
