use super::super::parameters::{PUNISH_SPATIAL_MIN_SAMPLES, PUNISH_SPATIAL_SAMPLE_PADDING};
use super::super::{DistanceBand, SpatialObservation};
use super::observations::reliable_actor_pair;
use crate::match_events::{PunishChance, PunishOutcome, PunishReachability};

pub(super) fn refine(punishes: &mut [PunishChance], observations: &[SpatialObservation]) {
    for punish in punishes {
        refine_one(punish, observations);
    }
}

fn refine_one(punish: &mut PunishChance, observations: &[SpatialObservation]) {
    if !matches!(
        punish.outcome,
        PunishOutcome::Missed | PunishOutcome::WhiffFail
    ) {
        return;
    }
    let sample_start = punish
        .source_contact_frame
        .unwrap_or(punish.frame)
        .saturating_sub(PUNISH_SPATIAL_SAMPLE_PADDING);
    let sample_end = punish
        .attack_active_frame
        .unwrap_or(punish.frame)
        .max(punish.frame)
        .saturating_add(PUNISH_SPATIAL_SAMPLE_PADDING);
    let bands: Vec<DistanceBand> = observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= sample_start && observation.frame_index <= sample_end
        })
        .filter_map(|observation| {
            reliable_actor_pair(observation)?;
            observation.distance_band
        })
        .collect();
    punish.reachability = reachability(punish.outcome, &bands);
}

fn reachability(outcome: PunishOutcome, bands: &[DistanceBand]) -> PunishReachability {
    let overlaps = count(bands, DistanceBand::Overlap);
    let close = count(bands, DistanceBand::Close);
    let mid = count(bands, DistanceBand::Mid);
    let far = count(bands, DistanceBand::Far);
    match outcome {
        // Without an attempted move, only overlapping bodies prove reachability.
        // Close and mid remain unknown because move-specific reach is unavailable.
        PunishOutcome::Missed => {
            if overlaps >= PUNISH_SPATIAL_MIN_SAMPLES && close + mid + far == 0 {
                PunishReachability::Confirmed
            } else if overlaps == 0 && mid + far >= PUNISH_SPATIAL_MIN_SAMPLES {
                PunishReachability::OutOfRange
            } else {
                PunishReachability::Unknown
            }
        }
        // A whiff candidate already has block and normal-active evidence. Stable
        // close-to-mid spacing is usable; only far spacing proves it out of range.
        PunishOutcome::WhiffFail => {
            if overlaps + close + mid >= PUNISH_SPATIAL_MIN_SAMPLES && far == 0 {
                PunishReachability::Confirmed
            } else if far >= PUNISH_SPATIAL_MIN_SAMPLES && overlaps + close + mid == 0 {
                PunishReachability::OutOfRange
            } else {
                PunishReachability::Unknown
            }
        }
        PunishOutcome::Success => PunishReachability::Confirmed,
    }
}

fn count(bands: &[DistanceBand], target: DistanceBand) -> usize {
    bands.iter().filter(|&&band| band == target).count()
}

#[cfg(test)]
mod tests;
