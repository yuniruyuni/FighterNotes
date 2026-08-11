use std::collections::HashSet;

use crate::model::TimelineEntry;

use super::super::MeterTracker;

impl MeterTracker {
    pub(super) fn finalize_until(&mut self, limit: i64) {
        let pending: HashSet<i64> = self
            .reads
            .get("left")
            .expect("left reads")
            .keys()
            .copied()
            .chain(self.dwell.keys().copied())
            .filter(|&absolute| absolute <= limit)
            .collect();
        let mut pending: Vec<i64> = pending.into_iter().collect();
        pending.sort_unstable();
        for absolute in pending {
            self.emit(absolute);
        }
    }

    fn emit(&mut self, absolute: i64) {
        let (video_frame_first, video_frame_last) = self
            .dwell
            .remove(&absolute)
            .map(|dwell| (dwell[0], dwell[1]))
            .unwrap_or((-1, -1));

        for side in ["left", "right"] {
            let (state, confidence, _) = self
                .reads
                .get_mut(side)
                .expect("side reads")
                .remove(&absolute)
                .unwrap_or_else(|| ("unknown".to_string(), 0.0, false));
            self.emitted
                .get_mut(side)
                .expect("side emitted reads")
                .extend([(absolute, state.clone())]);
            let timeline = if side == "left" {
                &mut self.left
            } else {
                &mut self.right
            };
            if let Some(segment) = timeline.segments.last_mut() {
                segment.entries.push(TimelineEntry {
                    game_frame: absolute,
                    state,
                    video_frame_first,
                    video_frame_last,
                    confidence: (confidence * 1000.0).round() / 1000.0,
                });
            }
        }
    }
}
