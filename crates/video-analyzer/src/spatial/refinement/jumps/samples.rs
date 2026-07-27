use super::super::super::parameters::{JUMP_AIR_MIN_SAMPLES, JUMP_AIR_SAMPLE_LOOKBACK};
use super::super::super::{ActorObservation, SpatialObservation};
use crate::match_events::{JumpEvent, JumpOutcome};

pub(super) fn actor_samples(
    observations: &[SpatialObservation],
    side: u8,
    start_frame: u32,
    end_frame: u32,
) -> Vec<(u32, &ActorObservation)> {
    observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= start_frame && observation.frame_index <= end_frame
        })
        .filter_map(|observation| {
            let actor = if side == 1 {
                observation.p1.as_ref()
            } else {
                observation.p2.as_ref()
            }?;
            (actor.observed && actor.confidence >= 0.45).then_some((observation.frame_index, actor))
        })
        .collect()
}

pub(super) fn refine_landed_hit(
    jump: &mut JumpEvent,
    samples: &[(u32, &ActorObservation)],
    contact_frame: u32,
) {
    let latest_airborne = samples
        .iter()
        .rev()
        .find_map(|(frame, actor)| (!actor.ground_anchor).then_some(*frame));
    let latest_grounded = samples
        .iter()
        .rev()
        .find_map(|(frame, actor)| actor.ground_anchor.then_some(*frame));
    let airborne_after_latest_ground = samples
        .iter()
        .filter(|(frame, actor)| {
            !actor.ground_anchor
                && latest_grounded.is_none_or(|grounded_frame| *frame > grounded_frame)
        })
        .count();
    let airborne_is_latest = latest_airborne.is_some_and(|airborne_frame| {
        latest_grounded.is_none_or(|grounded_frame| airborne_frame > grounded_frame)
            && airborne_frame.saturating_add(JUMP_AIR_SAMPLE_LOOKBACK) >= contact_frame
    });
    if airborne_after_latest_ground >= JUMP_AIR_MIN_SAMPLES && airborne_is_latest {
        jump.takeoff_confirmed = true;
    } else {
        jump.takeoff_confirmed = false;
        jump.outcome = JumpOutcome::Neutral;
    }
}

pub(super) fn refine_incoming_hit(jump: &mut JumpEvent, samples: &[(u32, &ActorObservation)]) {
    let airborne = samples
        .iter()
        .filter(|(_, actor)| !actor.ground_anchor)
        .count();
    let grounded = samples
        .iter()
        .filter(|(_, actor)| actor.ground_anchor)
        .count();
    if airborne >= JUMP_AIR_MIN_SAMPLES && airborne > grounded {
        jump.takeoff_confirmed = true;
        jump.outcome = JumpOutcome::GotHit;
        return;
    }
    jump.takeoff_confirmed = false;
    jump.outcome = if grounded >= JUMP_AIR_MIN_SAMPLES && grounded >= airborne {
        JumpOutcome::GroundedHit
    } else {
        // An observed window without stable airborne samples is not enough
        // evidence for user-facing anti-air advice.
        JumpOutcome::Neutral
    };
}
