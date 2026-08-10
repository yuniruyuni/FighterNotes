use wasm_bindgen::prelude::*;

use crate::serialization::hp_or_unknown;

use super::Analyzer;

#[wasm_bindgen]
impl Analyzer {
    /// Appends HP and drive-gauge features from the reusable HUD strip buffer.
    pub fn push_hud_features_inplace(
        &mut self,
        full_width: u32,
        full_height: u32,
        video_frame: u32,
    ) {
        if video_frame.is_multiple_of(video_analyzer::FIGHT_SAMPLE_INTERVAL) {
            self.fight_observations
                .push(video_analyzer::FightObservation {
                    frame: video_frame,
                    score: video_analyzer::fight_score_from_hud_strip(
                        &self.hud_buf,
                        full_width as usize,
                    ),
                });
            self.fight_markers = None;
        }
        let (raw_left, left_uncertain) = video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(
            &self.hud_buf,
            full_width,
            full_height,
            "p1",
        );
        let (raw_right, right_uncertain) =
            video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(
                &self.hud_buf,
                full_width,
                full_height,
                "p2",
            );
        let left_hp = hp_or_unknown(raw_left, left_uncertain);
        let right_hp = hp_or_unknown(raw_right, right_uncertain);
        let (own_hp, opponent_hp) = if self.own_side == "p1" {
            (left_hp, right_hp)
        } else {
            (right_hp, left_hp)
        };
        let left_hp_score = video_analyzer::hp_bar_score_from_hud_strip(
            &self.hud_buf,
            full_width,
            full_height,
            "p1",
        );
        let right_hp_score = video_analyzer::hp_bar_score_from_hud_strip(
            &self.hud_buf,
            full_width,
            full_height,
            "p2",
        );
        let left_drive = video_analyzer::drive_gauge_read_from_hud_strip(
            &self.hud_buf,
            full_width,
            full_height,
            "left",
        );
        let right_drive = video_analyzer::drive_gauge_read_from_hud_strip(
            &self.hud_buf,
            full_width,
            full_height,
            "right",
        );
        let left_super =
            video_analyzer::super_gauge_read_from_hud_strip(&self.hud_buf, full_width, "left");
        let right_super =
            video_analyzer::super_gauge_read_from_hud_strip(&self.hud_buf, full_width, "right");
        self.features.push(video_analyzer::FrameFeatures {
            frame_index: video_frame,
            fps: 60.0,
            own_hp,
            opponent_hp,
            is_match_screen: left_hp_score >= 0.035 && right_hp_score >= 0.025,
            own_meter_state: None,
            opponent_meter_state: None,
            left_hp_score,
            right_hp_score,
            left_drive_ratio: normalized_drive(&left_drive),
            right_drive_ratio: normalized_drive(&right_drive),
            left_burnout: left_drive.burnout,
            right_burnout: right_drive.burnout,
            left_drive_uncertain: left_drive.uncertain,
            right_drive_uncertain: right_drive.uncertain,
            left_super_value: left_super.value,
            right_super_value: right_super.value,
            left_super_uncertain: left_super.uncertain,
            right_super_uncertain: right_super.uncertain,
            left_ca_ready: ca_ready(&left_super, left_hp),
            right_ca_ready: ca_ready(&right_super, right_hp),
            left_hp_raw: raw_left,
            right_hp_raw: raw_right,
            left_hp_raw_quality: if left_uncertain { 1.0 } else { 0.0 },
            right_hp_raw_quality: if right_uncertain { 1.0 } else { 0.0 },
        });
        self.total_frames = video_frame + 1;
    }
}

fn normalized_drive(read: &video_analyzer::DriveGaugeRead) -> f32 {
    if read.burnout {
        read.recovery
    } else {
        read.value / 6.0
    }
}

fn ca_ready(read: &video_analyzer::SuperGaugeRead, hp: f32) -> bool {
    read.critical_art || (!read.uncertain && read.value >= 2.95 && (0.0..=0.255).contains(&hp))
}

#[cfg(test)]
mod tests;
