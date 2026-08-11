use super::super::super::SpatialObservation;
use super::super::observations::stable_distance_samples;
use super::direction::is_forward;
use crate::match_events::{DriveRushEvent, EventConfidence, InputSegment};

pub(super) fn refine(
    rushes: &mut [DriveRushEvent],
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    for rush in rushes {
        refine_one(rush, segments, observations);
    }
}

fn refine_one(
    rush: &mut DriveRushEvent,
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    let sample_end = rush.contact_frame.unwrap_or(rush.frame.saturating_add(60));
    let stable = stable_distance_samples(observations, rush.frame.saturating_sub(8), sample_end);
    let (Some(first), Some(last)) = (stable.first(), stable.last()) else {
        return;
    };
    let (Some(first_distance), Some(last_distance)) = (first.screen_distance, last.screen_distance)
    else {
        return;
    };
    let input_forward = segments[rush.side as usize - 1]
        .iter()
        .filter(|segment| segment.start_frame.abs_diff(rush.frame) <= 5)
        .any(|segment| is_forward(rush.side, &segment.dir, first.horizontal_order));
    if input_forward && first_distance - last_distance >= 0.04 {
        rush.confidence = EventConfidence::High;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_events::DriveRushOutcome;
    use crate::spatial::{
        ActorObservation, DistanceBand, HorizontalOrder, SpatialPoint, SpatialRect,
    };

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

    fn observation(frame_index: u32, distance: f32) -> SpatialObservation {
        SpatialObservation {
            frame_index,
            p1: Some(actor(0.2)),
            p2: Some(actor(0.2 + distance)),
            screen_distance: Some(distance),
            distance_band: Some(DistanceBand::Close),
            horizontal_order: Some(HorizontalOrder::P1Left),
            projectile_candidates: vec![],
            motion_regions: vec![],
        }
    }

    fn rush(contact_frame: Option<u32>) -> DriveRushEvent {
        DriveRushEvent {
            side: 1,
            frame: 10,
            raw: true,
            outcome: DriveRushOutcome::NoContact,
            contact_frame,
            damage: 0.0,
            confidence: EventConfidence::Medium,
            round_no: 1,
        }
    }

    fn forward_segment() -> InputSegment {
        InputSegment {
            start_frame: 10,
            end_frame: 10,
            dir: "R".into(),
            badges: vec![],
            auto: false,
            throw: false,
            evidence: Default::default(),
        }
    }

    #[test]
    fn fallback_window_uses_both_exact_edges_and_accepts_exact_closing_threshold() {
        let mut rushes = [rush(None)];
        let segments = [vec![forward_segment()], vec![]];
        refine(
            &mut rushes,
            &segments,
            &[
                observation(1, 0.0),
                observation(2, 0.04),
                observation(70, 0.0),
            ],
        );
        assert_eq!(rushes[0].confidence, EventConfidence::High);
    }

    #[test]
    fn forward_input_and_actual_closing_are_both_required() {
        let closing = [observation(10, 0.08), observation(20, 0.0)];
        let mut no_input = [rush(Some(20))];
        refine(&mut no_input, &[vec![], vec![]], &closing);
        assert_eq!(no_input[0].confidence, EventConfidence::Medium);

        let mut growing_distance = [rush(Some(20))];
        refine(
            &mut growing_distance,
            &[vec![forward_segment()], vec![]],
            &[observation(10, 0.02), observation(20, 0.06)],
        );
        assert_eq!(growing_distance[0].confidence, EventConfidence::Medium);
    }
}
