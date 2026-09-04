//! Round 開始直後の「側が確定している」区間のヒント。
//!
//! Round 開始時は両者が固定の左右へ配置され、初期間合いの広さから
//! しばらくは側の入れ替わりが物理的に起きない。この区間だけを
//! プレイヤーの色シグネチャの学習に使う。

use super::super::parameters::ROUND_OPEN_CERTAIN_FRAMES;
use super::model::{SpatialCandidateWindow, SpatialFrameRange};
use crate::match_events::RoundInfo;

pub(super) fn annotate(windows: &mut [SpatialCandidateWindow], rounds: &[RoundInfo]) {
    for window in windows.iter_mut() {
        for round in rounds {
            let start = round.start_frame;
            let end = round
                .start_frame
                .saturating_add(ROUND_OPEN_CERTAIN_FRAMES)
                .min(round.end_frame);
            if end < window.start_frame || start > window.end_frame {
                continue;
            }
            window.certain_side_hints.push(SpatialFrameRange {
                start_frame: start.max(window.start_frame),
                end_frame: end.min(window.end_frame),
            });
        }
        window
            .certain_side_hints
            .sort_by_key(|range| range.start_frame);
        window.certain_side_hints.dedup();
    }
}
