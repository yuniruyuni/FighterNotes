use super::model::SpatialCandidateWindow;
use super::round_bounds;
use crate::match_events::{PunishChance, PunishOutcome, PunishReachability, RoundInfo};
use crate::spatial::parameters::{PUNISH_SPATIAL_LOOKAHEAD, PUNISH_SPATIAL_LOOKBACK};

pub(super) fn windows(
    punishes: &[PunishChance],
    rounds: &[RoundInfo],
) -> Vec<SpatialCandidateWindow> {
    punishes
        .iter()
        .filter(|punish| {
            matches!(
                punish.outcome,
                PunishOutcome::Missed | PunishOutcome::WhiffFail
            ) && punish.reachability == PunishReachability::Unknown
        })
        .map(|punish| {
            let bounds = round_bounds::for_round(rounds, punish.round_no);
            SpatialCandidateWindow {
                start_frame: punish
                    .source_contact_frame
                    .unwrap_or(punish.frame)
                    .saturating_sub(PUNISH_SPATIAL_LOOKBACK)
                    .max(bounds.start),
                end_frame: punish
                    .attack_active_frame
                    .unwrap_or(punish.frame)
                    .max(punish.frame)
                    .saturating_add(PUNISH_SPATIAL_LOOKAHEAD)
                    .min(bounds.end),
                teleport_hints: vec![],
                airborne_hints: vec![],
            }
        })
        .collect()
}
