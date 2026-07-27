use super::model::SpatialCandidateWindow;
use super::round_bounds;
use crate::match_events::{
    DriveRushEvent, EventConfidence, RoundInfo, ThrowActionEvent, ThrowOutcome,
};

pub(super) fn throw_windows(
    throws: &[ThrowActionEvent],
    rounds: &[RoundInfo],
) -> Vec<SpatialCandidateWindow> {
    throws
        .iter()
        .filter(|event| {
            event.confidence == EventConfidence::High && event.outcome != ThrowOutcome::Unconfirmed
        })
        .map(|event| {
            let bounds = round_bounds::for_round(rounds, event.round_no);
            SpatialCandidateWindow {
                start_frame: event.input_frame.saturating_sub(45).max(bounds.start),
                end_frame: event
                    .active_frame
                    .unwrap_or(event.input_frame)
                    .saturating_add(30)
                    .min(bounds.end),
                teleport_hints: vec![],
                airborne_hints: vec![],
            }
        })
        .collect()
}

pub(super) fn drive_rush_windows(
    rushes: &[DriveRushEvent],
    rounds: &[RoundInfo],
) -> Vec<SpatialCandidateWindow> {
    rushes
        .iter()
        .map(|event| {
            let bounds = round_bounds::for_round(rounds, event.round_no);
            SpatialCandidateWindow {
                start_frame: event.frame.saturating_sub(15).max(bounds.start),
                end_frame: event
                    .contact_frame
                    .unwrap_or(event.frame.saturating_add(70))
                    .saturating_add(10)
                    .min(bounds.end),
                teleport_hints: vec![],
                airborne_hints: vec![],
            }
        })
        .collect()
}
