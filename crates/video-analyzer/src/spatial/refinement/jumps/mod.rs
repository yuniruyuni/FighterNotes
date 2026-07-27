mod direction;
mod samples;

use super::super::parameters::{
    JUMP_AIR_SAMPLE_LOOKBACK, JUMP_SPATIAL_LOOKAHEAD, JUMP_SPATIAL_LOOKBACK,
};
use super::super::SpatialObservation;
use super::observations::reliable_actor_pair;
use crate::match_events::{JumpDirection, JumpEvent, JumpOutcome};

pub(super) fn refine(jumps: &mut [JumpEvent], observations: &[SpatialObservation]) {
    for jump in jumps {
        if jump.direction == JumpDirection::Unknown {
            let order = observations
                .iter()
                .filter(|observation| observation.frame_index.abs_diff(jump.frame) <= 4)
                .find_map(|observation| {
                    reliable_actor_pair(observation)?;
                    observation.horizontal_order
                });
            jump.direction = direction::resolve(jump.side, &jump.input_dir, order);
        }

        if !matches!(
            jump.outcome,
            JumpOutcome::GotHit | JumpOutcome::UnverifiedHit | JumpOutcome::LandedHit
        ) {
            continue;
        }
        let Some(contact_frame) = jump.contact_frame else {
            continue;
        };
        let coverage_start = jump.frame.saturating_sub(JUMP_SPATIAL_LOOKBACK);
        let coverage_end = contact_frame.saturating_add(JUMP_SPATIAL_LOOKAHEAD);
        if !observations.iter().any(|observation| {
            observation.frame_index >= coverage_start && observation.frame_index <= coverage_end
        }) {
            // Refinement may receive observations for an unrelated event window.
            // Leave jumps that were not sampled untouched.
            continue;
        }
        let sample_start = if jump.outcome == JumpOutcome::LandedHit {
            coverage_start
        } else {
            contact_frame.saturating_sub(JUMP_AIR_SAMPLE_LOOKBACK)
        };
        let actor_samples =
            samples::actor_samples(observations, jump.side, sample_start, contact_frame);
        if jump.outcome == JumpOutcome::LandedHit {
            samples::refine_landed_hit(jump, &actor_samples, contact_frame);
        } else {
            samples::refine_incoming_hit(jump, &actor_samples);
        }
    }
}
