mod approaches;
mod corners;
mod distances;
mod jumps;
mod observations;
mod projectiles;
mod punishes;
mod teleports;

use super::parameters::{PERIODIC_SAMPLE_LEN, PERIODIC_SAMPLE_PERIOD};
use super::SpatialObservation;
use crate::context::AnalysisContext;
use crate::match_events::{MatchEvents, SpatialCoverage};
use std::collections::BTreeSet;

/// Adds spatial evidence without replacing input, meter, or contact evidence.
pub fn refine_match_events_with_spatial(
    events: &mut MatchEvents,
    observations: &[SpatialObservation],
    context: &AnalysisContext,
) {
    let windows = crate::spatial_decode_windows(events);
    let candidate_frames = windows
        .iter()
        .map(|window| {
            window
                .end_frame
                .saturating_sub(window.start_frame)
                .saturating_add(1)
        })
        .sum();
    let in_candidate_window = |frame: u32| {
        windows
            .iter()
            .any(|window| frame >= window.start_frame && frame <= window.end_frame)
    };
    let sampled: BTreeSet<_> = observations
        .iter()
        .filter(|observation| in_candidate_window(observation.frame_index))
        .map(|observation| observation.frame_index)
        .collect();
    let usable: BTreeSet<_> = observations
        .iter()
        .filter(|observation| in_candidate_window(observation.frame_index))
        .filter(|observation| {
            self::observations::reliable_actor_pair(observation).is_some()
                && observation.screen_distance.is_some()
        })
        .map(|observation| observation.frame_index)
        .collect();
    let observed_for = |side: u8| {
        observations
            .iter()
            .filter(|observation| in_candidate_window(observation.frame_index))
            .filter(|observation| {
                let actor = if side == 1 {
                    observation.p1.as_ref()
                } else {
                    observation.p2.as_ref()
                };
                actor.is_some_and(|actor| actor.observed)
            })
            .map(|observation| observation.frame_index)
            .collect::<BTreeSet<_>>()
            .len() as u32
    };
    // 時間比の分母になる周期サンプル。round 開始からの位相で window 計画と
    // 同じフレーム集合を復元する。イベント駆動 window と重なって復号された
    // フレームも、位相が合っていれば周期サンプルとして数えてよい。
    let is_periodic_sample = |frame: u32| {
        events.rounds.iter().any(|round| {
            frame >= round.start_frame
                && frame <= round.end_frame
                && (frame - round.start_frame) % PERIODIC_SAMPLE_PERIOD < PERIODIC_SAMPLE_LEN
        })
    };
    let mut periodic_pair_samples = 0;
    let mut cornered_samples = [0u32; 2];
    for observation in observations
        .iter()
        .filter(|observation| is_periodic_sample(observation.frame_index))
    {
        if self::observations::reliable_actor_pair(observation).is_none() {
            continue;
        }
        periodic_pair_samples += 1;
        if let Some((side, _)) = corners::cornered_side(observation) {
            cornered_samples[side as usize - 1] += 1;
        }
    }
    events.spatial_coverage = SpatialCoverage {
        candidate_frames,
        sampled_frames: sampled.len() as u32,
        usable_frames: usable.len() as u32,
        p1_observed_frames: observed_for(1),
        p2_observed_frames: observed_for(2),
        periodic_pair_samples,
        p1_cornered_samples: cornered_samples[0],
        p2_cornered_samples: cornered_samples[1],
    };

    jumps::refine(&mut events.jumps, observations);
    projectiles::refine(&mut events.projectiles, observations);
    punishes::refine(&mut events.punishes, observations);
    teleports::refine(
        &mut events.teleports,
        &events.segments,
        &events.meter_game_frame,
        observations,
        context,
    );
    projectiles::propagate_confidence(&events.projectiles, &mut events.compound_threats);
    approaches::refine_drive_rushes(&mut events.drive_rushes, &events.segments, observations);
    approaches::refine_throws(
        &mut events.throw_actions,
        &events.drive_rushes,
        &events.segments,
        observations,
    );
    events.corner_spans = corners::detect(observations);
    distances::attach(events, observations);
}
