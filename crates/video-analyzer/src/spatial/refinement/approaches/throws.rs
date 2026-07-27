use super::super::super::{DistanceBand, SpatialObservation};
use super::super::observations::stable_distance_samples;
use super::direction::is_forward;
use crate::match_events::{
    DriveRushEvent, EventConfidence, InputSegment, ThrowActionEvent, ThrowApproach, ThrowOutcome,
};

pub(super) fn refine(
    throws: &mut [ThrowActionEvent],
    rushes: &[DriveRushEvent],
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    for throw in throws {
        if throw.confidence != EventConfidence::High || throw.outcome != ThrowOutcome::Hit {
            continue;
        }
        if rushes.iter().any(|rush| {
            rush.side == throw.thrower
                && rush.confidence == EventConfidence::High
                && rush.frame <= throw.input_frame
                && rush.frame.saturating_add(90) >= throw.input_frame
        }) {
            throw.approach = ThrowApproach::DriveRush;
            continue;
        }
        refine_forward_dash(throw, segments, observations);
    }
}

fn refine_forward_dash(
    throw: &mut ThrowActionEvent,
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    let start = throw.input_frame.saturating_sub(42);
    let end = throw
        .active_frame
        .unwrap_or(throw.input_frame)
        .saturating_add(4);
    let stable = stable_distance_samples(observations, start, end);
    let (Some(first), Some(last)) = (stable.first(), stable.last()) else {
        return;
    };
    let (Some(first_distance), Some(last_distance)) = (first.screen_distance, last.screen_distance)
    else {
        return;
    };
    let final_close = matches!(
        last.distance_band,
        Some(DistanceBand::Overlap | DistanceBand::Close)
    );
    if !final_close || first_distance - last_distance < 0.04 {
        return;
    }
    let forward_inputs = segments[throw.thrower as usize - 1]
        .iter()
        .filter(|segment| segment.start_frame >= start && segment.start_frame <= throw.input_frame)
        .filter(|segment| {
            let order = observations
                .iter()
                .filter(|observation| observation.frame_index.abs_diff(segment.start_frame) <= 4)
                .find_map(|observation| observation.horizontal_order)
                .or(last.horizontal_order);
            is_forward(throw.thrower, &segment.dir, order)
        })
        .count();
    if forward_inputs >= 2 {
        throw.approach = ThrowApproach::ForwardDash;
    }
}
