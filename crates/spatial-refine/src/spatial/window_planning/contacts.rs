//! 第一段の contact イベントから hitstop のヒント区間を作る。
//!
//! meter と HP は接触の瞬間をフレーム精度で確定している。hitstop 中は
//! 両者とカメラが静止し、フレーム間差分に残るのはヒットエフェクト
//! だけになるため、この区間に限って明るく彩度の高い領域をスパークと
//! 読むことを抽出器へ許可する。

use super::super::parameters::CONTACT_HINT_TAIL_FRAMES;
use super::model::{SpatialCandidateWindow, SpatialFrameRange};
use crate::match_events::ContactEvent;

/// 各 window に、その範囲へかかる contact 区間を付与する。
pub(super) fn annotate(windows: &mut [SpatialCandidateWindow], contacts: &[ContactEvent]) {
    for window in windows.iter_mut() {
        for contact in contacts {
            let start = contact.frame;
            let end = contact.frame.saturating_add(CONTACT_HINT_TAIL_FRAMES);
            if end < window.start_frame || start > window.end_frame {
                continue;
            }
            window.contact_hints.push(SpatialFrameRange {
                start_frame: start.max(window.start_frame),
                end_frame: end.min(window.end_frame),
            });
        }
        window.contact_hints.sort_by_key(|range| range.start_frame);
        window.contact_hints.dedup();
    }
}
