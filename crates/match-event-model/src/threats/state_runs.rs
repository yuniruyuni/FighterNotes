use super::MeterTimeline;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub struct StateRun {
    pub start: u32,
    pub end: u32,
    pub distinct_game_frames: usize,
}

pub fn state_runs(timeline: &MeterTimeline, predicate: impl Fn(&str) -> bool) -> Vec<StateRun> {
    let mut spans: Vec<(u32, u32, i64)> = timeline
        .segments
        .iter()
        .flat_map(|segment| segment.entries.iter())
        .filter(|entry| entry.video_frame_first >= 0 && predicate(&entry.state))
        .map(|entry| {
            (
                entry.video_frame_first as u32,
                entry.video_frame_last.max(entry.video_frame_first) as u32,
                entry.game_frame,
            )
        })
        .collect();
    spans.sort_by_key(|span| span.0);

    let mut grouped: Vec<(StateRun, BTreeSet<i64>)> = Vec::new();
    for (start, end, game_frame) in spans {
        if let Some((last, game_frames)) = grouped.last_mut() {
            if start <= last.end.saturating_add(2) {
                last.end = last.end.max(end);
                game_frames.extend([game_frame]);
                last.distinct_game_frames = game_frames.len();
                continue;
            }
        }
        grouped.push((
            StateRun {
                start,
                end,
                distinct_game_frames: 1,
            },
            BTreeSet::from([game_frame]),
        ));
    }
    grouped.into_iter().map(|(run, _)| run).collect()
}
