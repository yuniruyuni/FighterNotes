use wasm_bindgen::prelude::*;

use super::Analyzer;

#[wasm_bindgen]
impl Analyzer {
    pub fn hud_buf_ptr(&self) -> u32 {
        self.hud_buf.as_ptr() as u32
    }

    pub fn hud_buf_len(&self) -> u32 {
        self.hud_buf.len() as u32
    }

    pub fn super_buf_ptr(&self) -> u32 {
        self.super_buf.as_ptr() as u32
    }

    pub fn super_buf_len(&self) -> u32 {
        self.super_buf.len() as u32
    }

    pub fn meter_buf_ptr(&self) -> u32 {
        self.meter_buf.as_ptr() as u32
    }

    pub fn meter_buf_len(&self) -> u32 {
        self.meter_buf.len() as u32
    }

    pub fn input_buf_ptr(&self) -> u32 {
        self.input_buf.as_ptr() as u32
    }

    pub fn input_buf_len(&self) -> u32 {
        self.input_buf.len() as u32
    }

    /// Reads each player's current input row from the reusable strip buffer.
    pub fn analyze_input_inplace(&mut self, full_width: u32, _full_height: u32, _video_frame: u32) {
        let p1 = video_analyzer::read_input_row0_from_strip(&self.input_buf, full_width, "p1");
        let p2 = video_analyzer::read_input_row0_from_strip(&self.input_buf, full_width, "p2");
        self.input_rows.push((p1, p2));
    }

    /// Updates the frame-meter tracker from the reusable meter strip buffer.
    pub fn analyze_meter_inplace(&mut self, full_width: u32, full_height: u32, video_frame: u32) {
        let (left, right) = frame_meter::extract_row_obs_from_strip_with_digit_hint(
            &self.meter_buf,
            full_width,
            full_height,
            self.tracker.digit_window_hint(),
        );
        self.tracker.update(video_frame as i64, left, right);
    }

    /// Reads the attack info windows from the reusable meter strip buffer.
    ///
    /// フレームメーターの追跡とは違い、読み取りは strip だけで決まる。実機
    /// 計測では 1 フレーム 1.1ms かかり meter ワーカーの費用の半分を占めて
    /// いたので、別のワーカーへ出せるよう分けてある。
    pub fn analyze_attack_info_inplace(&mut self, full_width: u32, video_frame: u32) {
        let attack_info =
            video_analyzer::read_attack_info_from_meter_strip(&self.meter_buf, full_width);
        self.attack_info_tracker.observe(video_frame, &attack_info);
    }
}
