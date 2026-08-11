use frame_meter::RowObs;

use crate::calibration::{CELL_COUNT_I64, RESET_DIVERGENCE};

use super::{MeterTracker, Shared};

impl MeterTracker {
    pub(crate) fn reset_replay(&mut self, edge: i64) {
        let window_start = self.window.len().saturating_sub(self.divergence as usize);
        let previous_absolute = self
            .window
            .get(window_start)
            .and_then(|entry| entry.prev_abs);

        if let Some(previous_absolute) = previous_absolute {
            self.dwell
                .retain(|&absolute, _| absolute <= previous_absolute);
            self.reads
                .get_mut("left")
                .expect("left reads")
                .retain(|&absolute, _| absolute <= previous_absolute);
            self.reads
                .get_mut("right")
                .expect("right reads")
                .retain(|&absolute, _| absolute <= previous_absolute);
            if let Some(dwell) = self.dwell.get_mut(&previous_absolute) {
                let first_window_frame = self.window[window_start].vf;
                if dwell[1] >= first_window_frame {
                    dwell[1] = dwell[0].max(first_window_frame - 1);
                }
            }
        }

        self.close_segment();
        let window_length = self.window.len() - window_start;
        let start_cell = (edge - (window_length as i64 - 1)).rem_euclid(CELL_COUNT_I64);
        self.open_segment(start_cell);

        let entries: Vec<(i64, Shared<RowObs>, Shared<RowObs>, bool)> = self.window[window_start..]
            .iter()
            .map(|entry| {
                (
                    entry.vf,
                    Shared::clone(&entry.left),
                    Shared::clone(&entry.right),
                    entry.vote_ok,
                )
            })
            .collect();
        for (index, (video_frame, left, right, vote_ok)) in entries.into_iter().enumerate() {
            if index > 0 {
                self.absolute_frame = self.absolute_frame.map(|absolute| absolute + 1);
            }
            self.record(video_frame, left.as_ref(), right.as_ref(), vote_ok, false);
        }
    }

    pub(crate) fn diverge_step(&mut self, edge: i64) -> bool {
        if let Some(previous_edge) = self.divergent_edge {
            if edge == (previous_edge + 1).rem_euclid(CELL_COUNT_I64) {
                self.divergence += 1;
            } else {
                self.divergence = 1;
            }
        } else {
            self.divergence = 1;
        }
        self.divergent_edge = Some(edge);
        self.divergence >= RESET_DIVERGENCE
    }
}
