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
        let sample_end = rush.contact_frame.unwrap_or(rush.frame.saturating_add(60));
        let stable =
            stable_distance_samples(observations, rush.frame.saturating_sub(8), sample_end);
        let (Some(first), Some(last)) = (stable.first(), stable.last()) else {
            continue;
        };
        let (Some(first_distance), Some(last_distance)) =
            (first.screen_distance, last.screen_distance)
        else {
            continue;
        };
        let input_forward = segments[rush.side as usize - 1]
            .iter()
            .filter(|segment| segment.start_frame.abs_diff(rush.frame) <= 5)
            .any(|segment| is_forward(rush.side, &segment.dir, first.horizontal_order));
        if input_forward && first_distance - last_distance >= 0.04 {
            rush.confidence = EventConfidence::High;
        }
    }
}
