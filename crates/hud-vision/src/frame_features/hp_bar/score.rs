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
            if let Some(pixel) = rgba.get(idx..).and_then(|bytes| bytes.first_chunk::<3>()) {
                let [_, s, v] = rgb_to_hsv(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32);
                if strictly_above(s, 45.0) && strictly_above(v, 80.0) {
                    match_count += 1;
                }
                total += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        match_count as f32 / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_frame(rgb: [u8; 3]) -> Vec<u8> {
        let mut rgba = vec![0u8; 192 * 108 * 4];
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[..3].copy_from_slice(&rgb);
            pixel[3] = 255;
        }
        rgba
    }

    #[test]
    fn score_requires_saturation_and_value_past_the_exact_edges() {
        assert_eq!(
            hp_bar_score(&solid_frame([255, 210, 210]), 192, 108, "p1"),
            0.0
        );
        assert_eq!(hp_bar_score(&solid_frame([80, 0, 0]), 192, 108, "p1"), 0.0);
        assert_eq!(hp_bar_score(&solid_frame([81, 0, 0]), 192, 108, "p1"), 1.0);
    }
}
