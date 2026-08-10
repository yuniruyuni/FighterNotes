use crate::calibration::{
    CELL_COUNT_I64, FREEZE_TIMEOUT, RESET_DIVERGENCE, RESYNC_TOLERANCE, WIPE_GUARD_MIN_CELLS,
};
use frame_meter::RowObs;

use super::{MeterTracker, Shared, WinEntry};

impl MeterTracker {
    /// 区間を閉じる道では `previous` を書き戻さない。`close_segment` が
    /// 直前フレームを捨てるので、次に開き直すときの参照は残らない。
    pub fn update(&mut self, video_frame: i64, left: RowObs, right: RowObs) {
        let left = Shared::new(left);
        let right = Shared::new(right);
        let edge = self.select_edge(left.fresh_edge, right.fresh_edge);
        let changed = self.changed_cells(&left, &right);
        let vote_ok = self.wipe_count(&left, &right) < WIPE_GUARD_MIN_CELLS;

        let previous_absolute = self.absolute_frame;
        self.window.push(WinEntry {
            vf: video_frame,
            left: Shared::clone(&left),
            right: Shared::clone(&right),
            vote_ok,
            prev_abs: previous_absolute,
        });
        let excess = self.window.len().saturating_sub(RESET_DIVERGENCE as usize);
        self.window.drain(..excess);

        if self.absolute_frame.is_none() {
            if edge >= 0 {
                if let Some(candidate) = self.open_candidate {
                    let candidate_delta = edge - candidate;
                    if matches!(candidate_delta, 0 | 1) {
                        self.open_candidate = None;
                        self.open_segment(candidate);
                        if self.window.len() >= 2 {
                            let previous = &self.window[self.window.len() - 2];
                            let (vf, left, right, vote_ok) = (
                                previous.vf,
                                Shared::clone(&previous.left),
                                Shared::clone(&previous.right),
                                previous.vote_ok,
                            );
                            self.record(vf, left.as_ref(), right.as_ref(), vote_ok, false);
                        }
                        if edge != candidate {
                            self.absolute_frame = Some(edge);
                        }
                        self.record(video_frame, left.as_ref(), right.as_ref(), vote_ok, false);
                    } else {
                        self.open_candidate = Some(edge);
                    }
                } else {
                    self.open_candidate = Some(edge);
                }
            } else {
                self.open_candidate = None;
            }
            self.previous = Some((left, right));
            return;
        }

        let absolute = self.absolute_frame.expect("active segment");
        let cell = absolute;
        let predicted = (absolute + 1).rem_euclid(CELL_COUNT_I64);
        let mut next_absolute = absolute;
        let edge_delta = (edge >= 0).then(|| Self::circ_delta(edge, cell));

        let reset = if let Some(delta) = edge_delta {
            if delta > 0 {
                if delta == 1 {
                    next_absolute = absolute + 1;
                    self.clear_divergence();
                    false
                } else if delta <= 1 + RESYNC_TOLERANCE {
                    next_absolute = absolute + delta;
                    self.clear_divergence();
                    false
                } else if self.diverge_step(edge) {
                    true
                } else {
                    if Self::near_front(&changed, predicted) {
                        next_absolute = absolute + 1;
                    }
                    false
                }
            } else if Self::all_blackish(&left) && Self::all_blackish(&right) {
                self.close_segment();
                return;
            } else if delta < 0 {
                if self.diverge_step(edge) {
                    true
                } else {
                    if vote_ok && Self::near_front(&changed, predicted) {
                        next_absolute = absolute + 1;
                    }
                    false
                }
            } else {
                self.clear_divergence();
                if vote_ok && Self::near_front(&changed, predicted) {
                    next_absolute = absolute + 1;
                }
                false
            }
        } else if Self::all_blackish(&left) && Self::all_blackish(&right) {
            self.close_segment();
            return;
        } else {
            self.clear_divergence();
            if vote_ok && Self::near_front(&changed, predicted) {
                next_absolute = absolute + 1;
            }
            false
        };

        if reset {
            self.reset_replay(edge);
            return;
        }

        if next_absolute == absolute {
            self.still_frames += 1;
            if self.still_frames >= FREEZE_TIMEOUT {
                self.close_segment();
                return;
            }
        } else {
            self.still_frames = 0;
        }

        let advanced = next_absolute != absolute;
        self.absolute_frame = Some(next_absolute);
        self.record(
            video_frame,
            left.as_ref(),
            right.as_ref(),
            vote_ok,
            advanced,
        );
        self.previous = Some((left, right));
    }

    fn clear_divergence(&mut self) {
        self.divergence = 0;
        self.divergent_edge = None;
    }
}
