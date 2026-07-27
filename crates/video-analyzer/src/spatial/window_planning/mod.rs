mod approaches;
mod jumps;
mod model;
mod punishes;
mod round_bounds;
mod teleports;

pub use model::{SpatialCandidateWindow, SpatialHintRange};

use crate::match_events::MatchEvents;

/// Builds candidate-driven second-pass windows from input and meter events.
pub fn spatial_candidate_windows(events: &MatchEvents) -> Vec<SpatialCandidateWindow> {
    let mut windows = teleports::windows(
        &events.teleports,
        &events.compound_threats,
        &events.jumps,
        &events.rounds,
    );
    windows.extend(punishes::windows(&events.punishes, &events.rounds));
    windows.extend(jumps::windows(&events.jumps, &events.rounds));
    windows.extend(approaches::throw_windows(
        &events.throw_actions,
        &events.rounds,
    ));
    windows.extend(approaches::drive_rush_windows(
        &events.drive_rushes,
        &events.rounds,
    ));
    merge_adjacent(windows)
}

fn merge_adjacent(mut windows: Vec<SpatialCandidateWindow>) -> Vec<SpatialCandidateWindow> {
    windows.sort_by_key(|window| window.start_frame);
    let mut merged: Vec<SpatialCandidateWindow> = Vec::new();
    for mut window in windows {
        if let Some(last) = merged.last_mut() {
            if window.start_frame <= last.end_frame.saturating_add(1) {
                last.end_frame = last.end_frame.max(window.end_frame);
                last.teleport_hints.append(&mut window.teleport_hints);
                last.airborne_hints.append(&mut window.airborne_hints);
                continue;
            }
        }
        merged.push(window);
    }
    for window in &mut merged {
        window
            .teleport_hints
            .sort_by_key(|hint| (hint.start_frame, hint.side));
        window.teleport_hints.dedup();
        window
            .airborne_hints
            .sort_by_key(|hint| (hint.start_frame, hint.side));
        window.airborne_hints.dedup();
    }
    merged
}
