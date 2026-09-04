mod direction;
mod samples;

use super::super::parameters::{
    CONTACT_AIRBORNE_MAX_Y, CONTACT_AIRBORNE_MIN_CONFIDENCE, CONTACT_HINT_TAIL_FRAMES,
    JUMP_AIR_SAMPLE_LOOKBACK, JUMP_SPATIAL_LOOKAHEAD, JUMP_SPATIAL_LOOKBACK,
};
use super::super::SpatialObservation;
use super::observations::reliable_actor_pair;
use crate::match_events::{JumpDirection, JumpEvent, JumpOutcome};

pub(super) fn refine(jumps: &mut [JumpEvent], observations: &[SpatialObservation]) {
    for jump in jumps {
        refine_one(jump, observations);
    }
}

fn refine_one(jump: &mut JumpEvent, observations: &[SpatialObservation]) {
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
        return;
    }
    let Some(contact_frame) = jump.contact_frame else {
        return;
    };
    let coverage_start = jump.frame.saturating_sub(JUMP_SPATIAL_LOOKBACK);
    let coverage_end = contact_frame.saturating_add(JUMP_SPATIAL_LOOKAHEAD);
    if !observations.iter().any(|observation| {
        observation.frame_index >= coverage_start && observation.frame_index <= coverage_end
    }) {
        // Refinement may receive observations for an unrelated event window.
        // Leave jumps that were not sampled untouched.
        return;
    }
    let sample_start = if jump.outcome == JumpOutcome::LandedHit {
        coverage_start
    } else {
        contact_frame.saturating_sub(JUMP_AIR_SAMPLE_LOOKBACK)
    };
    let actor_samples =
        samples::actor_samples(observations, jump.side, sample_start, contact_frame);
    let high_spark = high_contact_spark(observations, contact_frame);
    if jump.outcome == JumpOutcome::LandedHit {
        samples::refine_landed_hit(jump, &actor_samples, contact_frame, high_spark);
    } else {
        samples::refine_incoming_hit(jump, &actor_samples, high_spark);
    }
}

/// hitstop 中のスパークが立ち姿勢の頭より明確に上にあれば、その接触は
/// 空中の身体に当たっている。体の追跡が演出で切れた場面の傍証になる。
fn high_contact_spark(observations: &[SpatialObservation], contact_frame: u32) -> bool {
    observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= contact_frame
                && observation.frame_index <= contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES)
        })
        .filter_map(|observation| observation.contact.as_ref())
        .any(|contact| {
            contact.confidence >= CONTACT_AIRBORNE_MIN_CONFIDENCE
                && contact.center.y < CONTACT_AIRBORNE_MAX_Y
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{
        ActorObservation, DistanceBand, HorizontalOrder, SpatialPoint, SpatialRect,
    };

    fn actor() -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(0.3, 0.9),
            bounds: SpatialRect::new(0.27, 0.5, 0.33, 0.9),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    #[test]
    fn an_observation_before_the_exact_coverage_start_does_not_refine_the_jump() {
        let mut jumps = [JumpEvent {
            side: 1,
            frame: 10,
            outcome: JumpOutcome::GotHit,
            input_dir: "U".into(),
            direction: JumpDirection::Neutral,
            contact_frame: Some(20),
            takeoff_confirmed: true,
            air_end: 40,
            round_no: 1,
        }];
        let observations = [SpatialObservation {
            frame_index: 3,
            p1: Some(actor()),
            p2: Some(actor()),
            screen_distance: Some(0.2),
            distance_band: Some(DistanceBand::Close),
            horizontal_order: Some(HorizontalOrder::P1Left),
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        }];

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::GotHit);
        assert!(jumps[0].takeoff_confirmed);
    }
}
