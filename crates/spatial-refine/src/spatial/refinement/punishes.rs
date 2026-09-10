use super::super::parameters::{
    CONTACT_ACTOR_SAMPLE_LOOKBACK, CONTACT_AIRBORNE_MIN_CONFIDENCE, CONTACT_HINT_TAIL_FRAMES,
    HEIGHT_REFERENCE_MIN_ACTOR_HEIGHT, PUNISH_DEEP_CONTACT_MAX_REACH, PUNISH_SPATIAL_MIN_SAMPLES,
    PUNISH_SPATIAL_SAMPLE_PADDING,
};
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
    // 距離帯では断定できなかった見逃しでも、ガードスパークが攻撃側の体の
    // すぐそばにあれば技は根元で当たっている(めり込み)。両者は最速の
    // 反撃が届く距離にいたと確認できる。先端ガードは何も断定しない。
    if punish.outcome == PunishOutcome::Missed
        && punish.reachability == PunishReachability::Unknown
        && deep_blocked_contact(punish, observations)
    {
        punish.reachability = PunishReachability::Confirmed;
    }
}

/// ガードされた攻撃が根元で当たっていたか。ガードスパークの水平位置と
/// 攻撃側の anchor の距離を、攻撃側の身長単位(ズーム非依存)で測る。
fn deep_blocked_contact(punish: &PunishChance, observations: &[SpatialObservation]) -> bool {
    let Some(contact_frame) = punish.source_contact_frame else {
        return false;
    };
    let spark = observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= contact_frame
                && observation.frame_index <= contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES)
        })
        .filter_map(|observation| observation.contact.as_ref())
        .filter(|contact| contact.confidence >= CONTACT_AIRBORNE_MIN_CONFIDENCE)
        .max_by(|a, b| a.confidence.total_cmp(&b.confidence));
    let Some(spark) = spark else {
        return false;
    };
    let attacker = 3 - punish.side;
    let sample_start = contact_frame.saturating_sub(CONTACT_ACTOR_SAMPLE_LOOKBACK);
    let sample_end = contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES);
    let standing = observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= sample_start && observation.frame_index <= sample_end
        })
        .filter_map(|observation| {
            if attacker == 1 {
                observation.p1.as_ref()
            } else {
                observation.p2.as_ref()
            }
        })
        .rfind(|actor| actor.observed && actor.confidence >= 0.45 && actor.ground_anchor);
    let Some(standing) = standing else {
        return false;
    };
    let attacker_height = standing.bounds.bottom - standing.bounds.top;
    if attacker_height < HEIGHT_REFERENCE_MIN_ACTOR_HEIGHT {
        return false;
    }
    (spark.center.x - standing.anchor.x).abs() / attacker_height <= PUNISH_DEEP_CONTACT_MAX_REACH
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
