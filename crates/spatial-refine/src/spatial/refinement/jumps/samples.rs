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
    high_spark: bool,
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
    } else if high_spark {
        // 体の追跡が切れていても、頭上のスパークは空中での接触の傍証。
        jump.takeoff_confirmed = true;
    } else {
        jump.takeoff_confirmed = false;
        jump.outcome = JumpOutcome::Neutral;
    }
}

pub(super) fn refine_incoming_hit(
    jump: &mut JumpEvent,
    samples: &[(u32, &ActorObservation)],
    high_spark: bool,
) {
    // 第一段で「HP は読めないが、確認済み離陸の空中窓にメーター接触あり」
    // とした候補。SF6 は空中ガード不可なので、空間層が明確な接地を示さず
    // 演出で追跡不能な場合はヒットとして復元できる。
    let meter_air_contact = jump.outcome == JumpOutcome::UnverifiedHit && jump.takeoff_confirmed;
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
    if grounded >= JUMP_AIR_MIN_SAMPLES && grounded >= airborne {
        jump.takeoff_confirmed = false;
        jump.outcome = JumpOutcome::GroundedHit;
    } else if meter_air_contact {
        jump.outcome = JumpOutcome::GotHit;
    } else if high_spark {
        jump.takeoff_confirmed = true;
        jump.outcome = JumpOutcome::GotHit;
    } else {
        // An observed window without stable airborne samples is not enough
        // evidence for user-facing anti-air advice.
        jump.takeoff_confirmed = false;
        jump.outcome = JumpOutcome::Neutral;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_events::JumpDirection;
    use crate::spatial::{SpatialPoint, SpatialRect};

    fn actor(ground_anchor: bool) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(0.3, if ground_anchor { 0.9 } else { 0.6 }),
            bounds: SpatialRect::new(0.27, 0.4, 0.33, if ground_anchor { 0.9 } else { 0.7 }),
            confidence: 1.0,
            observed: true,
            ground_anchor,
            discontinuity: false,
        }
    }

    fn jump(outcome: JumpOutcome, takeoff_confirmed: bool) -> JumpEvent {
        JumpEvent {
            side: 1,
            frame: 10,
            outcome,
            input_dir: "U".into(),
            direction: JumpDirection::Neutral,
            contact_frame: Some(20),
            takeoff_confirmed,
            air_end: 40,
            round_no: 1,
        }
    }

    fn incoming(outcome: JumpOutcome, takeoff_confirmed: bool, grounded: &[bool]) -> JumpEvent {
        let actors: Vec<_> = grounded.iter().copied().map(actor).collect();
        let samples: Vec<_> = actors
            .iter()
            .enumerate()
            .map(|(index, actor)| (index as u32, actor))
            .collect();
        let mut jump = jump(outcome, takeoff_confirmed);
        refine_incoming_hit(&mut jump, &samples, false);
        jump
    }

    #[test]
    fn actor_sampling_uses_the_requested_side() {
        let observations = [SpatialObservation {
            frame_index: 15,
            p1: Some(actor(false)),
            p2: Some(actor(true)),
            screen_distance: None,
            distance_band: None,
            horizontal_order: None,
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        }];

        let p1 = actor_samples(&observations, 1, 15, 15);
        let p2 = actor_samples(&observations, 2, 15, 15);
        assert_eq!(p1.len(), 1);
        assert!(!p1[0].1.ground_anchor);
        assert_eq!(p2.len(), 1);
        assert!(p2[0].1.ground_anchor);
    }

    #[test]
    fn landed_hit_needs_the_exact_minimum_of_recent_airborne_samples() {
        let airborne = actor(false);
        let recent = [(19, &airborne), (20, &airborne)];
        let mut confirmed = jump(JumpOutcome::LandedHit, false);
        refine_landed_hit(&mut confirmed, &recent, 20, false);
        assert!(confirmed.takeoff_confirmed);
        assert_eq!(confirmed.outcome, JumpOutcome::LandedHit);

        let too_old = [(10, &airborne), (11, &airborne)];
        let mut rejected = jump(JumpOutcome::LandedHit, true);
        refine_landed_hit(&mut rejected, &too_old, 20, false);
        assert!(!rejected.takeoff_confirmed);
        assert_eq!(rejected.outcome, JumpOutcome::Neutral);
    }

    #[test]
    fn incoming_hit_requires_both_minimum_airborne_count_and_a_strict_majority() {
        let too_few = incoming(JumpOutcome::GotHit, true, &[false]);
        assert_eq!(too_few.outcome, JumpOutcome::Neutral);
        assert!(!too_few.takeoff_confirmed);

        let exact_minimum = incoming(JumpOutcome::GotHit, false, &[false, false]);
        assert_eq!(exact_minimum.outcome, JumpOutcome::GotHit);
        assert!(exact_minimum.takeoff_confirmed);

        let tied = incoming(JumpOutcome::GotHit, false, &[false, false, true, true]);
        assert_eq!(tied.outcome, JumpOutcome::GroundedHit);
        assert!(!tied.takeoff_confirmed);
    }

    #[test]
    fn grounded_and_meter_fallbacks_each_require_all_of_their_evidence() {
        let grounded = incoming(JumpOutcome::GotHit, false, &[true, true]);
        assert_eq!(grounded.outcome, JumpOutcome::GroundedHit);
        assert!(!grounded.takeoff_confirmed);

        let unconfirmed_meter = incoming(JumpOutcome::UnverifiedHit, false, &[]);
        assert_eq!(unconfirmed_meter.outcome, JumpOutcome::Neutral);
        assert!(!unconfirmed_meter.takeoff_confirmed);

        let wrong_outcome = incoming(JumpOutcome::GotHit, true, &[]);
        assert_eq!(wrong_outcome.outcome, JumpOutcome::Neutral);
        assert!(!wrong_outcome.takeoff_confirmed);

        let confirmed_meter = incoming(JumpOutcome::UnverifiedHit, true, &[]);
        assert_eq!(confirmed_meter.outcome, JumpOutcome::GotHit);
        assert!(confirmed_meter.takeoff_confirmed);
    }
}
