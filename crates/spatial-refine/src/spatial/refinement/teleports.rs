use super::super::{DistanceBand, SpatialObservation};
use super::observations::reliable_actor_pair;
use crate::context::AnalysisContext;
use crate::frame_data::{self, RisingReversalKind};
use crate::match_events::{DpReachability, InputSegment, TeleportContext, TeleportEvent};

const CHARGE_MIN_FRAMES: u32 = 45;
const CHARGE_RELEASE_GRACE: u32 = 10;

pub(super) fn refine(
    teleports: &mut [TeleportEvent],
    input_segments: &[Vec<InputSegment>; 2],
    meter_game_frame: &[Vec<i64>; 2],
    observations: &[SpatialObservation],
    context: &AnalysisContext,
) {
    for teleport in teleports {
        if teleport.context != TeleportContext::NakedAttack {
            continue;
        }
        let defender_character = context.player(teleport.defender).character.as_deref();
        let Some(reversal_kind) = defender_character.and_then(frame_data::rising_reversal_kind)
        else {
            continue;
        };
        if !rising_reversal_available(
            input_segments,
            meter_game_frame,
            teleport.defender,
            teleport.input_frame,
            reversal_kind,
        ) {
            continue;
        }
        let Some(target_frame) = teleport
            .followup_contact_frame
            .or(teleport.followup_attack_frame)
        else {
            continue;
        };
        let best = observations
            .iter()
            .filter(|observation| observation.frame_index.abs_diff(target_frame) <= 4)
            .filter_map(|observation| {
                let (p1, p2) = reliable_actor_pair(observation)?;
                Some((observation, p1.confidence + p2.confidence))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(observation, _)| observation);
        teleport.dp_reachability = match best.and_then(|observation| observation.distance_band) {
            Some(DistanceBand::Overlap) => DpReachability::Confirmed,
            Some(DistanceBand::Far) => DpReachability::OutOfRange,
            _ => DpReachability::Unknown,
        };
    }
}

fn rising_reversal_available(
    input_segments: &[Vec<InputSegment>; 2],
    meter_game_frame: &[Vec<i64>; 2],
    defender: u8,
    frame: u32,
    kind: RisingReversalKind,
) -> bool {
    if kind == RisingReversalKind::Motion {
        return true;
    }
    let segments = &input_segments[defender.saturating_sub(1) as usize];
    let Some((latest_index, latest)) = segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            segment.start_frame <= frame
                && is_down_direction(&segment.dir)
                && frame <= segment.end_frame.saturating_add(CHARGE_RELEASE_GRACE)
        })
        .max_by_key(|(_, segment)| segment.end_frame)
    else {
        return false;
    };
    let mut run_start = latest.start_frame;
    for previous in segments[..latest_index].iter().rev() {
        if previous.end_frame.saturating_add(2) < run_start || !is_down_direction(&previous.dir) {
            break;
        }
        run_start = previous.start_frame;
    }
    advancing_game_frames(
        &meter_game_frame[defender.saturating_sub(1) as usize],
        run_start,
        latest.end_frame.min(frame),
    ) >= CHARGE_MIN_FRAMES
}

fn advancing_game_frames(game_frames: &[i64], start: u32, end: u32) -> u32 {
    let start = start as usize;
    let end = end as usize;
    if end >= game_frames.len()
        || start > end
        || game_frames[start..=end]
            .iter()
            .any(|game_frame| *game_frame < 0)
    {
        return 0;
    }
    game_frames[start..=end]
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count() as u32
        + 1
}

fn is_down_direction(direction: &str) -> bool {
    matches!(direction, "D" | "DL" | "DR")
}

#[cfg(test)]
mod tests;
