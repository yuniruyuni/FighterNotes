use super::super::*;

/// HP ROI 内の各列がオレンジ色（ダメージ受け中）かどうかを返す。
///
/// SF6 のダメージ表現: ダメージを受けた瞬間その部分がオレンジ色になり、
/// 一定時間後に透明になる。
/// オレンジ判定: H=10–27, S>60, 80<V<200（OpenCV HSV 0-179）。
/// V<200 で低HP 時の黄色バー（V>200）を除外する。
pub fn hp_col_orange(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    hp_col_orange_impl(rgba, width, height, side, 0)
}

pub fn hp_col_orange_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> Vec<bool> {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_col_orange_impl(strip, full_width, full_height, side, y0)
}

pub(crate) fn hp_col_orange_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> Vec<bool> {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    if x1 >= x2 || y1 >= y2 {
        return Vec::new();
    }

    let roi_w = (x2 - x1) as usize;
    let roi_h = (y2 - y1) as usize;

    let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
    let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);
    let slope: f32 = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };

    let mut col_orange = vec![false; roi_w];
    for (cx, orange) in col_orange.iter_mut().enumerate() {
        let mut match_count = 0usize;
        let mut eff_count = 0usize;
        for ry in row_start..row_end {
            let x_offset = ((ry - row_start) as f32 * slope).round() as i32;
            let gx_i = x1 as i32 + cx as i32 + x_offset;
            if gx_i < x1 as i32 || gx_i >= x2 as i32 {
                continue;
            }
            let gx = gx_i as usize;
            let gy = y1 as usize + ry;
            let idx = ((gy - y_strip_start) * width as usize + gx) * 4;
            if idx + 2 >= rgba.len() {
                continue;
            }
            eff_count += 1;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;
            let [h_hsv, s, v] = rgb_to_hsv(r, g, b);
            // classify_hp_col と同一の修正: V 上限なしで高輝度オレンジも捕捉
            if (10.0..=27.0).contains(&h_hsv) && s > 60.0 && v > 80.0 {
                match_count += 1;
            }
        }
        if eff_count > 0 && (match_count as f32 / eff_count as f32) > 0.15 {
            *orange = true;
        }
    }
    col_orange
}

/// HP ROI 内の各列が黄色（低 HP: 残量 25% 以下）かどうかを返す。
///
/// SF6: HP が 25% 以下になるとバーが赤/青から黄色に変化する。
/// 黄色判定: H=22–35, S>120, V>200（OpenCV HSV 0-179）。
pub fn hp_col_yellow(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    hp_col_yellow_impl(rgba, width, height, side, 0)
}

pub fn hp_col_yellow_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> Vec<bool> {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_col_yellow_impl(strip, full_width, full_height, side, y0)
}

pub(crate) fn hp_col_yellow_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> Vec<bool> {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    if x1 >= x2 || y1 >= y2 {
        return Vec::new();
    }

    let roi_w = (x2 - x1) as usize;
    let roi_h = (y2 - y1) as usize;

    let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
    let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);
    let slope: f32 = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };

    let mut col_yellow = vec![false; roi_w];
    for (cx, yellow) in col_yellow.iter_mut().enumerate() {
        let mut match_count = 0usize;
        let mut eff_count = 0usize;
        for ry in row_start..row_end {
            let x_offset = ((ry - row_start) as f32 * slope).round() as i32;
            let gx_i = x1 as i32 + cx as i32 + x_offset;
            if gx_i < x1 as i32 || gx_i >= x2 as i32 {
                continue;
            }
            let gx = gx_i as usize;
            let gy = y1 as usize + ry;
            let idx = ((gy - y_strip_start) * width as usize + gx) * 4;
            if idx + 2 >= rgba.len() {
                continue;
            }
            eff_count += 1;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;
            let [h_hsv, s, v] = rgb_to_hsv(r, g, b);
            if (22.0..=35.0).contains(&h_hsv) && s > 120.0 && v > 200.0 {
                match_count += 1;
            }
        }
        if eff_count > 0 && (match_count as f32 / eff_count as f32) > 0.15 {
            *yellow = true;
        }
    }
    col_yellow
}
