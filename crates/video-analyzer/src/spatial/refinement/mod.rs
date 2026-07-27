mod approaches;
mod jumps;
mod observations;
mod projectiles;
mod punishes;
mod teleports;

use super::SpatialObservation;
use crate::context::AnalysisContext;
use crate::match_events::MatchEvents;

/// Adds spatial evidence without replacing input, meter, or contact evidence.
pub fn refine_match_events_with_spatial(
    events: &mut MatchEvents,
    observations: &[SpatialObservation],
    context: &AnalysisContext,
) {
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
