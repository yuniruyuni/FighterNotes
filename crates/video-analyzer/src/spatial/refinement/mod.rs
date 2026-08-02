mod approaches;
mod jumps;
mod observations;
mod projectiles;
mod punishes;
mod teleports;

use super::SpatialObservation;
use crate::context::AnalysisContext;
use crate::match_events::{MatchEvents, SpatialCoverage};
use std::collections::BTreeSet;

/// Adds spatial evidence without replacing input, meter, or contact evidence.
pub fn refine_match_events_with_spatial(
    events: &mut MatchEvents,
    observations: &[SpatialObservation],
    context: &AnalysisContext,
) {
    let windows = crate::spatial_candidate_windows(events);
    let candidate_frames = windows
        .iter()
        .map(|window| {
            window
                .end_frame
                .saturating_sub(window.start_frame)
                .saturating_add(1)
        })
        .sum();
    let in_candidate_window = |frame: u32| {
        windows
            .iter()
            .any(|window| frame >= window.start_frame && frame <= window.end_frame)
    };
    let sampled: BTreeSet<_> = observations
        .iter()
        .filter(|observation| in_candidate_window(observation.frame_index))
        .map(|observation| observation.frame_index)
        .collect();
    let usable: BTreeSet<_> = observations
        .iter()
        .filter(|observation| in_candidate_window(observation.frame_index))
        .filter(|observation| {
            self::observations::reliable_actor_pair(observation).is_some()
                && observation.screen_distance.is_some()
        })
        .map(|observation| observation.frame_index)
        .collect();
    let observed_for = |side: u8| {
        observations
            .iter()
            .filter(|observation| in_candidate_window(observation.frame_index))
            .filter(|observation| {
                let actor = if side == 1 {
                    observation.p1.as_ref()
                } else {
                    observation.p2.as_ref()
                };
                actor.is_some_and(|actor| actor.observed)
            })
            .map(|observation| observation.frame_index)
            .collect::<BTreeSet<_>>()
            .len() as u32
    };
    events.spatial_coverage = SpatialCoverage {
        candidate_frames,
        sampled_frames: sampled.len() as u32,
        usable_frames: usable.len() as u32,
        p1_observed_frames: observed_for(1),
        p2_observed_frames: observed_for(2),
    };

    jumps::refine(&mut events.jumps, observations);
    projectiles::refine(&mut events.projectiles, observations);
    punishes::refine(&mut events.punishes, observations);
    teleports::refine(
        &mut events.teleports,
        &events.segments,
        &events.meter_game_frame,
        observations,
        context,
    );
    projectiles::propagate_confidence(&events.projectiles, &mut events.compound_threats);
    approaches::refine_drive_rushes(&mut events.drive_rushes, &events.segments, observations);
    approaches::refine_throws(
        &mut events.throw_actions,
        &events.drive_rushes,
        &events.segments,
        observations,
    );
}
