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
        // GPU が数える取り決めなら、ここでは走査しない。値は解析の最後に
        // まとめて入る。
        let (left_hp_score, right_hp_score) = if self.hp_scores_come_from_gpu {
            (0.0, 0.0)
        } else {
            (
                video_analyzer::hp_bar_score_from_hud_strip(
                    &self.hud_buf,
                    full_width,
                    full_height,
                    "p1",
                ),
                video_analyzer::hp_bar_score_from_hud_strip(
                    &self.hud_buf,
                    full_width,
                    full_height,
                    "p2",
                ),
            )
        };
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

    /// HP スコアの画素数えを GPU 側へ任せると決める。
    pub fn use_gpu_hp_scores(&mut self) {
        self.hp_scores_come_from_gpu = true;
    }

    /// GPU が引く画素判定表を返す。索引は `max * 256 + min`。
    ///
    /// 判定に使う彩度と明度の計算を GPU でやり直すと、除算の丸めが処理系
    /// 依存になる。参照実装で表を作り、GPU には索引だけをさせる。
    pub fn hp_score_table() -> Vec<u8> {
        video_analyzer::hp_score_decision_table()
    }

    /// GPU へ渡す走査範囲を `[p1_x1, p1_y1, p1_x2, p1_y2, p2_...]` で返す。
    pub fn hp_score_rois() -> Vec<u32> {
        let (a, b, c, d) = video_analyzer::hp_score_roi_in_strip("p1");
        let (e, f, g, h) = video_analyzer::hp_score_roi_in_strip("p2");
        vec![a, b, c, d, e, f, g, h]
    }

    /// GPU が数えた画素数から HP スコアを入れる。
    ///
    /// 並びは 1 フレームあたり `[p1_一致, p1_全体, p2_一致, p2_全体]`。割り算は
    /// 走査していた頃と同じ式で行い、試合画面かどうかも入れ直す。
    pub fn apply_hp_score_counts(&mut self, counts: &[u32]) -> Result<(), JsValue> {
        self.apply_hp_score_counts_impl(counts)
            .map_err(|error| JsValue::from_str(&error))
    }
}

impl Analyzer {
    pub(crate) fn apply_hp_score_counts_impl(&mut self, counts: &[u32]) -> Result<(), String> {
        if counts.len() != self.features.len() * 4 {
            return Err(format!(
                "hp score counts mismatch: expected {} values, got {}",
                self.features.len() * 4,
                counts.len()
            ));
        }
        for (feature, counted) in self.features.iter_mut().zip(counts.chunks_exact(4)) {
            feature.left_hp_score = ratio(counted[0], counted[1]);
            feature.right_hp_score = ratio(counted[2], counted[3]);
            feature.is_match_screen =
                feature.left_hp_score >= 0.035 && feature.right_hp_score >= 0.025;
        }
        Ok(())
    }
}

fn ratio(matched: u32, total: u32) -> f32 {
    if total == 0 {
        0.0
    } else {
        matched as f32 / total as f32
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
