use super::*;

/// RGBA バッファから HP バーの存在スコアを返す（0.0–1.0）。
///
/// 鮮やかで明るい画素の割合。色相は問わない。試合画面かどうかの判定に
/// 使うので、どの色のバーでも同じように反応する必要がある。
pub fn hp_bar_score(rgba: &[u8], width: u32, height: u32, side: &str) -> f32 {
    hp_bar_score_impl(rgba, width, height, side, 0)
}

pub fn hp_bar_score_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> f32 {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_bar_score_impl(strip, full_width, full_height, side, y0)
}

pub(crate) fn hp_bar_score_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> f32 {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);

    let mut match_count = 0u32;
    let mut total = 0u32;
    for gy in y1..y2 {
        for gx in x1..x2 {
            let idx = ((gy as usize - y_strip_start) * width as usize + gx as usize) * 4;
            if idx + 2 >= rgba.len() {
                continue;
            }
            let [_, s, v] =
                rgb_to_hsv(rgba[idx] as f32, rgba[idx + 1] as f32, rgba[idx + 2] as f32);
            if s > 45.0 && v > 80.0 {
                match_count += 1;
            }
            total += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        match_count as f32 / total as f32
    }
}
