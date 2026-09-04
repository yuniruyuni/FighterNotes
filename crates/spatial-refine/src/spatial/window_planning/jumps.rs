use super::model::{SpatialCandidateWindow, SpatialHintRange};
use super::round_bounds;
use crate::match_events::{JumpEvent, JumpOutcome, RoundInfo, JUMP_C_PRE_MAX};
use crate::spatial::parameters::{JUMP_SPATIAL_LOOKAHEAD, JUMP_SPATIAL_LOOKBACK};

pub(super) fn windows(jumps: &[JumpEvent], rounds: &[RoundInfo]) -> Vec<SpatialCandidateWindow> {
    jumps
        .iter()
        .filter(|jump| {
            matches!(
                jump.outcome,
                JumpOutcome::GotHit | JumpOutcome::UnverifiedHit | JumpOutcome::LandedHit
            )
        })
        .map(|jump| {
            let bounds = round_bounds::for_round(rounds, jump.round_no);
            let contact = jump.contact_frame.unwrap_or(jump.air_end);
            SpatialCandidateWindow {
                start_frame: jump
                    .frame
                    .saturating_sub(JUMP_SPATIAL_LOOKBACK)
                    .max(bounds.start),
                end_frame: contact
                    .saturating_add(JUMP_SPATIAL_LOOKAHEAD)
                    .min(bounds.end),
                teleport_hints: vec![],
                airborne_hints: vec![SpatialHintRange {
                    side: jump.side,
                    start_frame: jump.frame.saturating_add(JUMP_C_PRE_MAX + 1),
                    end_frame: contact.min(jump.air_end),
                }],
                contact_hints: vec![],
                certain_side_hints: vec![],
            }
        })
        .collect()
}
