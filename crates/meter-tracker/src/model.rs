use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub game_frame: i64,
    pub state: String,
    pub video_frame_first: i64,
    pub video_frame_last: i64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSegment {
    pub segment_id: i32,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterTimeline {
    pub side: String,
    pub segments: Vec<TimelineSegment>,
}

impl MeterTimeline {
    /// Returns a stun flag for each video frame.
    pub fn stun_per_frame(&self, total_frames: u32) -> Vec<bool> {
        let mut result = vec![false; total_frames as usize];
        for segment in &self.segments {
            for entry in &segment.entries {
                if entry.state != "stun" {
                    continue;
                }
                let first = entry.video_frame_first.max(0) as usize;
                let end = (entry.video_frame_last.max(0) as usize + 1).min(total_frames as usize);
                for flag in &mut result[first.min(end)..end] {
                    *flag = true;
                }
            }
        }
        result
    }
}
