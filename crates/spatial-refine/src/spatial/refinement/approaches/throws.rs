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
        refine_one(throw, rushes, segments, observations);
    }
}

fn refine_one(
    throw: &mut ThrowActionEvent,
    rushes: &[DriveRushEvent],
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    if !matches!(
        (throw.confidence, throw.outcome),
        (EventConfidence::High, ThrowOutcome::Hit)
    ) {
        return;
    }
    if rushes.iter().any(|rush| {
        rush.side == throw.thrower
            && rush.confidence == EventConfidence::High
            && rush.frame <= throw.input_frame
            && rush.frame.saturating_add(90) >= throw.input_frame
    }) {
        throw.approach = ThrowApproach::DriveRush;
        return;
    }
    refine_forward_dash(throw, segments, observations);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_events::DriveRushOutcome;
    use crate::spatial::{ActorObservation, HorizontalOrder, SpatialPoint, SpatialRect};

    fn throw(confidence: EventConfidence, outcome: ThrowOutcome) -> ThrowActionEvent {
        ThrowActionEvent {
            thrower: 1,
            input_frame: 100,
            startup_frame: Some(102),
            active_frame: Some(105),
            outcome,
            damage: 0.0,
            approach: ThrowApproach::Unknown,
            confidence,
            round_no: 1,
        }
    }

    fn actor(x: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(x, 0.9),
            bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.9),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    fn observation(
        frame_index: u32,
        distance: f32,
        distance_band: DistanceBand,
    ) -> SpatialObservation {
        SpatialObservation {
            frame_index,
            p1: Some(actor(0.2)),
            p2: Some(actor(0.2 + distance)),
            screen_distance: Some(distance),
            distance_band: Some(distance_band),
            horizontal_order: Some(HorizontalOrder::P1Left),
            projectile_candidates: vec![],
            motion_regions: vec![],
        }
    }

    fn forward_segment(frame: u32) -> InputSegment {
        InputSegment {
            start_frame: frame,
            end_frame: frame,
            dir: "R".into(),
            badges: vec![],
            auto: false,
            throw: false,
            evidence: Default::default(),
        }
    }

    #[test]
    fn only_a_high_confidence_hit_is_eligible_and_later_throws_are_still_refined() {
        let mut throws = [
            throw(EventConfidence::Low, ThrowOutcome::Hit),
            throw(EventConfidence::High, ThrowOutcome::Teched),
            throw(EventConfidence::High, ThrowOutcome::Hit),
        ];
        let rushes = [DriveRushEvent {
            side: 1,
            frame: 50,
            raw: true,
            outcome: DriveRushOutcome::Blocked,
            contact_frame: Some(90),
            damage: 0.0,
            confidence: EventConfidence::High,
            round_no: 1,
        }];

        refine(&mut throws, &rushes, &[vec![], vec![]], &[]);

        assert_eq!(throws[0].approach, ThrowApproach::Unknown);
        assert_eq!(throws[1].approach, ThrowApproach::Unknown);
        assert_eq!(throws[2].approach, ThrowApproach::DriveRush);
    }

    #[test]
    fn forward_dash_uses_the_exact_lookback_and_closing_threshold() {
        let mut throws = [throw(EventConfidence::High, ThrowOutcome::Hit)];
        let segments = [vec![forward_segment(58), forward_segment(75)], vec![]];
        refine(
            &mut throws,
            &[],
            &segments,
            &[
                observation(57, 0.0, DistanceBand::Close),
                observation(58, 0.04, DistanceBand::Close),
                observation(109, 0.0, DistanceBand::Overlap),
            ],
        );
        assert_eq!(throws[0].approach, ThrowApproach::ForwardDash);
    }

    #[test]
    fn forward_dash_requires_a_close_ending_and_a_decreasing_distance() {
        let segments = [vec![forward_segment(60), forward_segment(75)], vec![]];

        let mut ends_mid = [throw(EventConfidence::High, ThrowOutcome::Hit)];
        refine(
            &mut ends_mid,
            &[],
            &segments,
            &[
                observation(60, 0.10, DistanceBand::Mid),
                observation(105, 0.02, DistanceBand::Mid),
            ],
        );
        assert_eq!(ends_mid[0].approach, ThrowApproach::Unknown);

        let mut moving_apart = [throw(EventConfidence::High, ThrowOutcome::Hit)];
        refine(
            &mut moving_apart,
            &[],
            &segments,
            &[
                observation(60, 0.02, DistanceBand::Close),
                observation(105, 0.06, DistanceBand::Close),
            ],
        );
        assert_eq!(moving_apart[0].approach, ThrowApproach::Unknown);
    }
}
