mod direction;
mod drive_rushes;
mod throws;

use super::SpatialObservation;
use crate::match_events::{DriveRushEvent, InputSegment, ThrowActionEvent};

pub(super) fn refine_drive_rushes(
    rushes: &mut [DriveRushEvent],
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    drive_rushes::refine(rushes, segments, observations);
}

pub(super) fn refine_throws(
    throws: &mut [ThrowActionEvent],
    rushes: &[DriveRushEvent],
    segments: &[Vec<InputSegment>; 2],
    observations: &[SpatialObservation],
) {
    throws::refine(throws, rushes, segments, observations);
}
