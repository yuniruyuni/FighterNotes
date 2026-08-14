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
    super_buf: Vec<u8>,
    fight_observations: Vec<video_analyzer::FightObservation>,
    fight_markers: Option<Vec<video_analyzer::FightMarker>>,
    require_fight_markers: bool,
    meter_buf: Vec<u8>,
    attack_info_tracker: video_analyzer::AttackInfoTracker,
    hp_scores_come_from_gpu: bool,
    hp_fills_come_from_gpu: bool,
    drive_comes_from_gpu: bool,
    hp_fills: Vec<(f32, bool, f32, bool)>,
    ca_gates: Vec<(bool, bool, bool, bool)>,
    hp_score_counts: Vec<[u32; 4]>,
    drive_reads: Vec<(
        video_analyzer::DriveGaugeRead,
        video_analyzer::DriveGaugeRead,
    )>,
    input_buf: Vec<u8>,
    input_rows: Vec<(video_analyzer::InputRow, video_analyzer::InputRow)>,
    tracked_json: Option<String>,
    analysis_context: video_analyzer::AnalysisContext,
    imported_meter: Option<(meter_tracker::MeterTimeline, meter_tracker::MeterTimeline)>,
    imported_attack_info: Option<Vec<video_analyzer::AttackInfoObservation>>,
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
            super_buf: vec![0; STRIP_WIDTH * video_analyzer::SUPER_STRIP_H as usize * 4],
            fight_observations: Vec::new(),
            fight_markers: None,
            require_fight_markers: false,
            meter_buf: vec![0; STRIP_WIDTH * METER_STRIP_H as usize * 4],
            attack_info_tracker: video_analyzer::AttackInfoTracker::default(),
            hp_scores_come_from_gpu: false,
            hp_fills_come_from_gpu: false,
            drive_comes_from_gpu: false,
            hp_fills: Vec::new(),
            ca_gates: Vec::new(),
            hp_score_counts: Vec::new(),
            drive_reads: Vec::new(),
            input_buf: vec![0; STRIP_WIDTH * video_analyzer::INPUT_STRIP_H as usize * 4],
            input_rows: Vec::new(),
            tracked_json: None,
            analysis_context: video_analyzer::AnalysisContext::new(own_side),
            imported_meter: None,
            imported_attack_info: None,
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
