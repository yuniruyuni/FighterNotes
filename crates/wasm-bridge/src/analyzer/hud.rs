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
        // GPU が列を分類する取り決めなら、充填率も後からまとめて入る。
        let ((raw_left, left_uncertain), (raw_right, right_uncertain)) =
            if self.hp_fills_come_from_gpu {
                ((0.0, false), (0.0, false))
            } else {
                (
                    video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(
                        &self.hud_buf,
                        full_width,
                        full_height,
                        "p1",
                    ),
                    video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(
                        &self.hud_buf,
                        full_width,
                        full_height,
                        "p2",
                    ),
                )
            };
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
        // GPU が列を分類する取り決めなら、ゲージの値も後からまとめて入る。
        let (left_drive, right_drive) = if self.drive_comes_from_gpu {
            (unread_drive(), unread_drive())
        } else {
            (
                video_analyzer::drive_gauge_read_from_hud_strip(
                    &self.hud_buf,
                    full_width,
                    full_height,
                    "left",
                ),
                video_analyzer::drive_gauge_read_from_hud_strip(
                    &self.hud_buf,
                    full_width,
                    full_height,
                    "right",
                ),
            )
        };
        // SA ゲージは等倍で置いた帯から読む。縮小を挟まないので画素を落とさない。
        let left_super =
            video_analyzer::super_gauge_read_from_native_strip(&self.super_buf, full_width, "left");
        let right_super = video_analyzer::super_gauge_read_from_native_strip(
            &self.super_buf,
            full_width,
            "right",
        );
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
        if self.hp_fills_come_from_gpu {
            // CA の判定は HP を見る。HP を後から入れる以上、SA ゲージ側の
            // 条件だけを控えておき、揃った時点で判定し直す。
            self.ca_gates.push((
                left_super.critical_art,
                ca_gate(&left_super),
                right_super.critical_art,
                ca_gate(&right_super),
            ));
        }
        self.total_frames = video_frame + 1;
    }

    /// HP スコアの画素数えを GPU 側へ任せると決める。
    pub fn use_gpu_hp_scores(&mut self) {
        self.hp_scores_come_from_gpu = true;
    }

    /// いま持っている strip から、CPU 側で分類した列の色を返す。
    /// GPU の答えと突き合わせるために使う。
    pub fn hp_columns_from_strip(&self, side: &str) -> Vec<u8> {
        video_analyzer::hp_columns_from_strip(&self.hud_buf, side)
    }

    /// HP の充填率も GPU が分類した列から求めると決める。
    pub fn use_gpu_hp_columns(&mut self) {
        self.hp_fills_come_from_gpu = true;
    }

    /// ドライブゲージも GPU が分類した列から求めると決める。
    pub fn use_gpu_drive(&mut self) {
        self.drive_comes_from_gpu = true;
    }

    /// GPU へ渡すドライブゲージの走査の形。
    pub fn drive_column_scan(side: &str) -> Vec<u32> {
        video_analyzer::drive_column_scan(side)
    }

    /// いま持っている strip から、CPU 側で分類したドライブの列を返す。
    pub fn drive_columns_from_strip(&self, side: &str) -> Vec<u8> {
        video_analyzer::drive_columns_from_strip(&self.hud_buf, side)
    }

    /// GPU が分類したドライブの列を受け取る。並びは左・右の順。
    pub fn apply_drive_columns(
        &mut self,
        first_frame: u32,
        columns: &[u32],
    ) -> Result<(), JsValue> {
        self.apply_drive_columns_impl(first_frame, columns)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// GPU が分類した列を受け取り、フレームごとの充填率にする。
    ///
    /// 並びは 1 フレームあたり p1・p2 の順。まとまりごとに届くので、
    /// 先頭のフレーム番号で置き場所を決める。
    pub fn apply_hp_columns(&mut self, first_frame: u32, columns: &[u32]) -> Result<(), JsValue> {
        self.apply_hp_columns_impl(first_frame, columns)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// GPU が引く、チャンネル値を 0..1 へ正規化した表。
    pub fn channel_norm_table() -> Vec<f32> {
        video_analyzer::channel_norm_table()
    }

    /// GPU が引く彩度と明度の表。索引は `max * 256 + min`、値は `[s, v]`。
    pub fn hsv_sv_table() -> Vec<f32> {
        video_analyzer::hsv_sv_table()
    }

    /// GPU へ渡す列走査の形。`[x1, roi_w, strip_y1, row_start, row_end, 右下がりか]`。
    pub fn hp_column_scan(side: &str) -> Vec<u32> {
        video_analyzer::hp_column_scan(side)
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

    /// GPU が数えた画素数を、まとまりごとに受け取る。
    ///
    /// 先頭のフレーム番号で置き場所を決める。特徴量へ入れるのは解析の最後。
    pub fn push_hp_score_counts(
        &mut self,
        first_frame: u32,
        counts: &[u32],
    ) -> Result<(), JsValue> {
        self.push_hp_score_counts_impl(first_frame, counts)
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

impl Analyzer {
    pub(crate) fn push_hp_score_counts_impl(
        &mut self,
        first_frame: u32,
        counts: &[u32],
    ) -> Result<(), String> {
        if !counts.len().is_multiple_of(4) {
            return Err(format!(
                "hp score counts must arrive in whole frames of 4 values, got {}",
                counts.len()
            ));
        }
        for (offset, frame_counts) in counts.chunks_exact(4).enumerate() {
            let frame = first_frame as usize + offset;
            if self.hp_score_counts.len() <= frame {
                // 届かなかったフレームは「数えていない」ままにする。
                self.hp_score_counts.resize(frame + 1, [0, 0, 0, 0]);
            }
            self.hp_score_counts[frame] = [
                frame_counts[0],
                frame_counts[1],
                frame_counts[2],
                frame_counts[3],
            ];
        }
        Ok(())
    }

    pub(crate) fn apply_drive_columns_impl(
        &mut self,
        first_frame: u32,
        columns: &[u32],
    ) -> Result<(), String> {
        let width = video_analyzer::drive_column_scan("left")[1] as usize;
        let per_frame = width * 2;
        if per_frame == 0 || !columns.len().is_multiple_of(per_frame) {
            return Err(format!(
                "drive columns must arrive in whole frames of {per_frame} values, got {}",
                columns.len()
            ));
        }
        for (offset, frame_columns) in columns.chunks_exact(per_frame).enumerate() {
            let codes: Vec<u8> = frame_columns.iter().map(|&code| code as u8).collect();
            let frame = first_frame as usize + offset;
            if self.drive_reads.len() <= frame {
                // 届かなかったフレームは「読めなかった」にしておく。
                self.drive_reads
                    .resize(frame + 1, (unread_drive(), unread_drive()));
            }
            let (left_codes, right_codes) = codes.split_at(width);
            self.drive_reads[frame] = (
                video_analyzer::drive_read_from_columns(left_codes, "left"),
                video_analyzer::drive_read_from_columns(right_codes, "right"),
            );
        }
        Ok(())
    }

    pub(crate) fn apply_hp_columns_impl(
        &mut self,
        first_frame: u32,
        columns: &[u32],
    ) -> Result<(), String> {
        let width = video_analyzer::hp_column_scan("p1")[1] as usize;
        let per_frame = width * 2;
        if per_frame == 0 || !columns.len().is_multiple_of(per_frame) {
            return Err(format!(
                "hp columns must arrive in whole frames of {per_frame} values, got {}",
                columns.len()
            ));
        }
        for (offset, frame_columns) in columns.chunks_exact(per_frame).enumerate() {
            let codes: Vec<u8> = frame_columns.iter().map(|&code| code as u8).collect();
            let frame = first_frame as usize + offset;
            if self.hp_fills.len() <= frame {
                // 届かなかったフレームは「読めなかった」にしておく。0% と
                // 言い切ると、欠けた区間がそのまま瀕死として扱われる。
                self.hp_fills.resize(frame + 1, (0.0, true, 0.0, true));
            }
            let (left_codes, right_codes) = codes.split_at(width);
            let (left, left_uncertain) =
                video_analyzer::hp_fill_ratio_from_columns(left_codes, "p1");
            let (right, right_uncertain) =
                video_analyzer::hp_fill_ratio_from_columns(right_codes, "p2");
            self.hp_fills[frame] = (left, left_uncertain, right, right_uncertain);
        }
        Ok(())
    }

    /// 受け取った充填率とスコアを特徴量へ入れる。
    pub(crate) fn apply_hp_fills(&mut self) -> Result<(), String> {
        if self.hp_scores_come_from_gpu && !self.hp_score_counts.is_empty() {
            let counts: Vec<u32> = self.hp_score_counts.concat();
            self.apply_hp_score_counts_impl(&counts)?;
        }
        if self.drive_comes_from_gpu {
            if self.drive_reads.len() != self.features.len() {
                return Err(format!(
                    "drive reads mismatch: expected {} frames, got {}",
                    self.features.len(),
                    self.drive_reads.len()
                ));
            }
            for (feature, (left, right)) in self.features.iter_mut().zip(self.drive_reads.iter()) {
                feature.left_drive_ratio = normalized_drive(left);
                feature.right_drive_ratio = normalized_drive(right);
                feature.left_burnout = left.burnout;
                feature.right_burnout = right.burnout;
                feature.left_drive_uncertain = left.uncertain;
                feature.right_drive_uncertain = right.uncertain;
            }
        }
        if !self.hp_fills_come_from_gpu {
            return Ok(());
        }
        if self.hp_fills.len() != self.features.len() {
            return Err(format!(
                "hp fills mismatch: expected {} frames, got {}",
                self.features.len(),
                self.hp_fills.len()
            ));
        }
        if self.ca_gates.len() != self.features.len() {
            return Err(format!(
                "ca gates mismatch: expected {} frames, got {}",
                self.features.len(),
                self.ca_gates.len()
            ));
        }
        let own_is_p1 = self.own_side == "p1";
        for ((feature, &(left, left_uncertain, right, right_uncertain)), &gates) in self
            .features
            .iter_mut()
            .zip(self.hp_fills.iter())
            .zip(self.ca_gates.iter())
        {
            let left_hp = hp_or_unknown(left, left_uncertain);
            let right_hp = hp_or_unknown(right, right_uncertain);
            (feature.own_hp, feature.opponent_hp) = if own_is_p1 {
                (left_hp, right_hp)
            } else {
                (right_hp, left_hp)
            };
            feature.left_hp_raw = left;
            feature.right_hp_raw = right;
            feature.left_hp_raw_quality = if left_uncertain { 1.0 } else { 0.0 };
            feature.right_hp_raw_quality = if right_uncertain { 1.0 } else { 0.0 };
            let (left_critical, left_gate, right_critical, right_gate) = gates;
            feature.left_ca_ready = left_critical || (left_gate && hp_is_low(left_hp));
            feature.right_ca_ready = right_critical || (right_gate && hp_is_low(right_hp));
        }
        Ok(())
    }
}

/// まだ読み取りが届いていないゲージ。
fn unread_drive() -> video_analyzer::DriveGaugeRead {
    video_analyzer::DriveGaugeRead {
        value: 0.0,
        burnout: false,
        recovery: 0.0,
        uncertain: true,
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
    read.critical_art || (ca_gate(read) && hp_is_low(hp))
}

/// CA 判定のうち、SA ゲージだけで決まる部分。
fn ca_gate(read: &video_analyzer::SuperGaugeRead) -> bool {
    !read.uncertain && read.value >= 2.95
}

fn hp_is_low(hp: f32) -> bool {
    (0.0..=0.255).contains(&hp)
}

#[cfg(test)]
mod tests;
