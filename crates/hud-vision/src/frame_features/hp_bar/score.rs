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

/// `hp_bar_score_impl` の画素判定を、明るさの最大値と最小値だけで引ける表にする。
///
/// 判定に使う彩度と明度は 3 チャンネルの最大値と最小値だけで決まる。GPU 側で
/// 同じ判定をさせると f32 の除算精度が処理系依存になるため、参照実装である
/// この関数で表を作り、GPU には整数の索引だけをさせる。
///
/// 索引は `max * 256 + min`。
pub fn hp_score_decision_table() -> Vec<u8> {
    let mut table = vec![0u8; 256 * 256];
    for max in 0..256usize {
        for min in 0..=max {
            // 最大値と最小値さえ同じなら、どの画素でも彩度と明度は同じ値になる。
            let [_, s, v] = rgb_to_hsv(max as f32, min as f32, min as f32);
            if strictly_above(s, 45.0) && strictly_above(v, 80.0) {
                table[max * 256 + min] = 1;
            }
        }
    }
    table
}

/// GPU へ渡す、strip 内での HP スコア走査範囲 (x1, y1, x2, y2)。
pub fn hp_score_roi_in_strip(side: &str) -> (u32, u32, u32, u32) {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, 1920, 1080);
    (x1, y1 - HUD_STRIP_Y, x2, y2 - HUD_STRIP_Y)
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

    /// 表は画素走査の言い換えでなければならない。ずれていると GPU 側だけが
    /// 違う答えを出し、しかも見た目には気づけない。
    #[test]
    fn the_decision_table_answers_exactly_as_the_pixel_scan_does() {
        let table = hp_score_decision_table();

        for rgb in [
            [255, 210, 210],
            [80, 0, 0],
            [81, 0, 0],
            [204, 100, 100],
            [205, 100, 100],
            [120, 119, 118],
            [0, 0, 0],
            [255, 255, 255],
        ] {
            let scanned = hp_bar_score(&solid_frame(rgb), 192, 108, "p1");
            let max = rgb.iter().copied().max().expect("3 チャンネル") as usize;
            let min = rgb.iter().copied().min().expect("3 チャンネル") as usize;
            let looked_up = f32::from(table[max * 256 + min]);

            assert_eq!(scanned, looked_up, "{rgb:?} で答えが違う");
        }
    }

    /// 走査範囲は strip の先頭からの座標で渡す。フレーム全体の座標のままだと
    /// GPU 側が 64 行ずれたところを読む。
    #[test]
    fn the_roi_is_given_in_strip_coordinates() {
        let (_, y1, _, y2) = hp_score_roi_in_strip("p1");

        assert_eq!((y1, y2), (0, 95 - HUD_STRIP_Y));
    }
}
