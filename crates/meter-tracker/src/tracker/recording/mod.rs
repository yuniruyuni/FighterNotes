mod emission;
mod lifecycle;

use frame_meter::RowObs;

use crate::calibration::{
    CELL_COUNT_I64, DIM_READ_POSITIONS, READ_DIM_CONF, READ_EARLY_CONF, READ_FADE_CONF,
    READ_FRESH_CONF, READ_SETTLE_OFFSET, READ_WINDOW,
};

use super::MeterTracker;

impl MeterTracker {
    pub(crate) fn record(
        &mut self,
        video_frame: i64,
        left: &RowObs,
        right: &RowObs,
        vote_ok: bool,
        advanced: bool,
    ) {
        let absolute = self.absolute_frame.expect("active segment");
        self.video_map
            .insert(video_frame, (self.segment_id, absolute));
        self.dwell
            .entry(absolute)
            .and_modify(|dwell| dwell[1] = video_frame)
            .or_insert([video_frame, video_frame]);

        if vote_ok {
            let lap = absolute / CELL_COUNT_I64;
            let cell = absolute % CELL_COUNT_I64;
            for (side, observation) in [("left", left), ("right", right)] {
                let previous = self.previous.as_ref().map(|(left, right)| {
                    if side == "left" {
                        left.clone()
                    } else {
                        right.clone()
                    }
                });

                let raw_state = observation.states[cell as usize].as_str().to_string();
                let covered = Self::digit_covered(observation, cell);
                let rescued = observation.rescued[cell as usize];
                let (state, covered) = if (raw_state == "other" || rescued) && advanced {
                    (
                        self.resolve_slab(observation, previous.as_ref(), cell, side),
                        false,
                    )
                } else {
                    (raw_state, covered)
                };
                self.store_read(side, absolute, state, READ_FADE_CONF, covered);

                for offset in 1..=READ_WINDOW {
                    let target = absolute - offset;
                    if target < 0 || target / CELL_COUNT_I64 != lap {
                        break;
                    }
                    let target_cell = target % CELL_COUNT_I64;
                    let state = observation.states[target_cell as usize]
                        .as_str()
                        .to_string();
                    let covered = Self::digit_covered(observation, target_cell);
                    let confidence = if offset >= READ_SETTLE_OFFSET {
                        READ_FRESH_CONF
                    } else {
                        READ_EARLY_CONF
                    };
                    self.store_read(side, target, state, confidence, covered);
                }

                if lap > 0 && cell <= READ_WINDOW {
                    for &position in DIM_READ_POSITIONS {
                        let target = (lap - 1) * CELL_COUNT_I64 + position;
                        if target < absolute - READ_WINDOW {
                            continue;
                        }
                        if let Some(state) = Self::dim_read(observation, position) {
                            self.store_read(side, target, state, READ_DIM_CONF, false);
                        }
                    }
                }
            }
        }
        self.finalize_until(absolute - READ_WINDOW - 1);
    }

    fn store_read(
        &mut self,
        side: &str,
        absolute: i64,
        state: String,
        confidence: f64,
        covered: bool,
    ) {
        let current = self.reads.get(side).expect("side reads").get(&absolute);
        if Self::better_read(current, &state, confidence, covered) {
            self.reads
                .get_mut(side)
                .expect("side reads")
                .insert(absolute, (state, confidence, covered));
        }
    }
}
