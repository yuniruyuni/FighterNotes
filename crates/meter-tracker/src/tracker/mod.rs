mod observation;
mod reading;
mod recording;
mod replay;
mod update;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use frame_meter::RowObs;

use crate::model::MeterTimeline;

#[cfg(target_arch = "wasm32")]
pub(crate) type Shared<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Shared<T> = std::sync::Arc<T>;

pub(crate) type ReadEntry = (String, f64, bool);

pub(crate) struct WinEntry {
    pub(crate) vf: i64,
    pub(crate) left: Shared<RowObs>,
    pub(crate) right: Shared<RowObs>,
    pub(crate) vote_ok: bool,
    pub(crate) prev_abs: Option<i64>,
}

pub struct MeterTracker {
    pub left: MeterTimeline,
    pub right: MeterTimeline,
    /// Maps a video frame to `(segment_id, absolute_game_frame)`.
    pub video_map: HashMap<i64, (i32, i64)>,

    pub(crate) segment_id: i32,
    pub(crate) absolute_frame: Option<i64>,
    pub(crate) reads: HashMap<String, HashMap<i64, ReadEntry>>,
    pub(crate) dwell: HashMap<i64, [i64; 2]>,
    pub(crate) emitted: HashMap<String, HashMap<i64, String>>,
    pub(crate) previous: Option<(Shared<RowObs>, Shared<RowObs>)>,
    pub(crate) divergence: i64,
    pub(crate) still_frames: i64,
    pub(crate) open_candidate: Option<i64>,
    pub(crate) divergent_edge: Option<i64>,
    pub(crate) window: Vec<WinEntry>,
}

impl MeterTracker {
    pub fn new() -> Self {
        let reads = HashMap::from([
            (String::from("left"), HashMap::new()),
            (String::from("right"), HashMap::new()),
        ]);
        let emitted = HashMap::from([
            (String::from("left"), HashMap::new()),
            (String::from("right"), HashMap::new()),
        ]);
        Self {
            left: MeterTimeline {
                side: String::from("left"),
                segments: vec![],
            },
            right: MeterTimeline {
                side: String::from("right"),
                segments: vec![],
            },
            video_map: HashMap::new(),
            segment_id: -1,
            absolute_frame: None,
            reads,
            dwell: HashMap::new(),
            emitted,
            previous: None,
            divergence: 0,
            still_frames: 0,
            open_candidate: None,
            divergent_edge: None,
            window: Vec::new(),
        }
    }

    pub fn finish(&mut self) {
        self.close_segment();
    }

    pub fn close(&mut self) {
        self.close_segment();
    }

    pub fn suspend(&mut self) {
        self.previous = None;
    }

    /// Returns the current circular cell and the number of prior cells that
    /// frame-meter digit recognition may need on the next update.
    pub fn digit_window_hint(&self) -> Option<(usize, usize)> {
        self.absolute_frame.map(|absolute| {
            (
                absolute.rem_euclid(crate::calibration::CELL_COUNT_I64) as usize,
                crate::calibration::READ_WINDOW as usize,
            )
        })
    }
}

impl Default for MeterTracker {
    fn default() -> Self {
        Self::new()
    }
}
