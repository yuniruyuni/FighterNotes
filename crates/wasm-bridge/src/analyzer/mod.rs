mod buffers;
mod context;
mod hud;
mod results;

#[cfg(test)]
mod tests;

use frame_meter::METER_STRIP_H;
use video_analyzer::HUD_STRIP_H;
use wasm_bindgen::prelude::*;

const STRIP_WIDTH: usize = 1920;

/// Incremental browser-side video analysis session.
#[wasm_bindgen]
pub struct Analyzer {
    tracker: meter_tracker::MeterTracker,
    features: Vec<video_analyzer::FrameFeatures>,
    own_side: String,
    total_frames: u32,
    hud_buf: Vec<u8>,
    meter_buf: Vec<u8>,
    input_buf: Vec<u8>,
    input_rows: Vec<(video_analyzer::InputRow, video_analyzer::InputRow)>,
    tracked_json: Option<String>,
    analysis_context: video_analyzer::AnalysisContext,
    imported_meter: Option<(meter_tracker::MeterTimeline, meter_tracker::MeterTimeline)>,
    imported_timeline_json: Option<String>,
    events: Option<video_analyzer::MatchEvents>,
}

#[wasm_bindgen]
impl Analyzer {
    /// Creates an analyzer for `"p1"` or `"p2"`.
    #[wasm_bindgen(constructor)]
    pub fn new(own_side: &str) -> Analyzer {
        Analyzer {
            tracker: meter_tracker::MeterTracker::new(),
            features: Vec::new(),
            own_side: own_side.to_string(),
            total_frames: 0,
            hud_buf: vec![0; STRIP_WIDTH * HUD_STRIP_H as usize * 4],
            meter_buf: vec![0; STRIP_WIDTH * METER_STRIP_H as usize * 4],
            input_buf: vec![0; STRIP_WIDTH * video_analyzer::INPUT_STRIP_H as usize * 4],
            input_rows: Vec::new(),
            tracked_json: None,
            analysis_context: video_analyzer::AnalysisContext::new(own_side),
            imported_meter: None,
            imported_timeline_json: None,
            events: None,
        }
    }

    pub fn progress(&self) -> f32 {
        if self.total_frames == 0 {
            0.0
        } else {
            1.0
        }
    }
}
