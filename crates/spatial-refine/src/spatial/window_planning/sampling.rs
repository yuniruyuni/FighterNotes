//! 薄いサンプリング window。
//!
//! イベント駆動の window は攻防の瞬間へ偏るため、そこから「割合」や
//! 「傾向」を出すと嘘になる。damage 起点の短い window は被弾直前の
//! 間合いの観測に、round 内の定周期 window は端滞在時間のような時間比の
//! 観測に使う。どちらもヒントを持たない素の観測区間である。

use super::super::parameters::{
    DAMAGE_DISTANCE_LOOKBACK, PERIODIC_SAMPLE_LEN, PERIODIC_SAMPLE_PERIOD,
};
use super::model::SpatialCandidateWindow;
use super::round_bounds;
use crate::match_events::{DamageEvent, RoundInfo};

fn bare_window(start_frame: u32, end_frame: u32) -> SpatialCandidateWindow {
    SpatialCandidateWindow {
        start_frame,
        end_frame,
        teleport_hints: vec![],
        airborne_hints: vec![],
        contact_hints: vec![],
        certain_side_hints: vec![],
    }
}

/// 被弾直前の間合いを観測する window。演出フリーズがあれば、その前から
/// 遡る(フリーズ中の画は間合いの証拠にならない)。
pub(super) fn damage_windows(
    damage: &[DamageEvent],
    rounds: &[RoundInfo],
) -> Vec<SpatialCandidateWindow> {
    damage
        .iter()
        .map(|event| {
            let bounds = round_bounds::for_round(rounds, event.round_no);
            bare_window(
                event
                    .pre_freeze_frame
                    .saturating_sub(DAMAGE_DISTANCE_LOOKBACK)
                    .max(bounds.start),
                event.pre_freeze_frame.min(bounds.end),
            )
        })
        .collect()
}

/// round 内を一定間隔で薄く観測する window。round 境界の外は覗かない。
pub(super) fn periodic_windows(rounds: &[RoundInfo]) -> Vec<SpatialCandidateWindow> {
    let mut windows = Vec::new();
    for round in rounds {
        for start in (round.start_frame..=round.end_frame).step_by(PERIODIC_SAMPLE_PERIOD as usize)
        {
            windows.push(bare_window(
                start,
                start
                    .saturating_add(PERIODIC_SAMPLE_LEN - 1)
                    .min(round.end_frame),
            ));
        }
    }
    windows
}
