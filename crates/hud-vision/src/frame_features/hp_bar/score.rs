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

/// GPU が分類した列の色から HP の充填率を読む。
///
/// 色は `HpColColor` の並び順の番号で受け取る。範囲外の値は「空き」にする。
/// 画素を分類するところだけが GPU 側へ移り、並びの読み方は変わらない。
pub fn hp_fill_ratio_from_columns(columns: &[u8], side: &str) -> (f32, bool) {
    let hue = if side == "p1" {
        HpFillHue::Red
    } else {
        HpFillHue::Blue
    };
    let colors: Vec<HpColColor> = columns.iter().map(|&code| hp_col_color(code)).collect();
    let decode = decode_from_columns(colors, columns.len(), hue);
    (decode.fill_ratio, decode.uncertain)
}

/// GPU へ渡す列走査の形。
///
/// `[x1, roi_w, strip_y1, row_start, row_end]` と、傾きが右下がりかどうか。
/// 走査する行と斜めのずらし方を GPU 側と一致させるために使う。
pub fn hp_column_scan(side: &str) -> Vec<u32> {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, 1920, 1080);
    let roi_h = (y2 - y1) as usize;
    let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
    let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);
    vec![
        x1,
        x2 - x1,
        y1 - HUD_STRIP_Y,
        row_start as u32,
        row_end as u32,
        u32::from(side == "p1"),
    ]
}

fn hp_col_color(code: u8) -> HpColColor {
    match code {
        0 => HpColColor::White,
        1 => HpColColor::Fill,
        2 => HpColColor::Ghost,
        3 => HpColColor::YellowWhite,
        4 => HpColColor::Orange,
        _ => HpColColor::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// HUD strip 1 枚分。
    fn solid_strip(rgb: [u8; 3]) -> Vec<u8> {
        let mut rgba = vec![0u8; 1920 * 70 * 4];
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[..3].copy_from_slice(&rgb);
            pixel[3] = 255;
        }
        rgba
    }

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

    /// 列の色から読んだ充填率は、画素から読んだものと同じでなければ
    /// ならない。GPU 側へ分類を移しても答えが変わらないことの土台になる。
    /// 端の白枠・充填・充填端の白線・遠端の白枠、という並びを読む。
    fn capped_columns(roi_w: usize, fill_end: usize) -> Vec<u8> {
        let mut columns = vec![HpColColor::Dark as u8; roi_w];
        columns[0..3].fill(HpColColor::White as u8);
        columns[3..fill_end].fill(HpColColor::Fill as u8);
        columns[fill_end..fill_end + 2].fill(HpColColor::White as u8);
        columns[roi_w - 3..].fill(HpColColor::White as u8);
        columns
    }

    /// 白いキャップから充填が続く並びを、GPU が出す列の色として渡す。
    #[test]
    fn a_capped_run_of_fill_reads_as_that_share_of_the_bar() {
        let roi_w = hp_column_scan("p2")[1] as usize;
        let columns = capped_columns(roi_w, roi_w / 2);

        let (ratio, uncertain) = hp_fill_ratio_from_columns(&columns, "p2");

        assert!(!uncertain, "読めなかった");
        assert!((0.45..=0.55).contains(&ratio), "充填が {ratio} になった");
    }

    /// 同じ並びでも左右で読む向きが違う。取り違えると充填率が逆になる。
    #[test]
    fn the_two_sides_read_the_same_columns_from_opposite_ends() {
        let roi_w = hp_column_scan("p1")[1] as usize;
        let columns = capped_columns(roi_w, roi_w / 2);

        let p1 = hp_fill_ratio_from_columns(&columns, "p1");
        let p2 = hp_fill_ratio_from_columns(&columns, "p2");

        assert_ne!(p1, p2, "左右で同じ答えになっている");
    }

    /// 列が来なければ読めない。空を「充填 0」と読むと、解析の頭が
    /// まるごと満タン扱いになる。
    #[test]
    fn no_columns_at_all_cannot_be_read() {
        let (ratio, _) = hp_fill_ratio_from_columns(&[], "p2");

        assert_eq!(ratio, 0.0);
    }

    /// ROI の列は一本残らず色が付く。数が足りないと、その分だけ
    /// 充填率が短く出る。
    #[test]
    fn every_column_of_the_roi_gets_a_colour() {
        let strip = solid_strip([220, 40, 40]);

        let columns =
            super::super::decode::classify_columns(&strip, 1920, 1080, "p2", HUD_STRIP_Y as usize);

        assert_eq!(columns.len(), hp_column_scan("p2")[1] as usize);
        assert!(
            columns.iter().all(|color| *color == columns[0]),
            "一面の同じ色が列ごとに違う色へ分かれている"
        );
    }

    /// 画素から分類した列は、画素から直接読んだ答えと一致する。
    #[test]
    fn reading_from_columns_matches_reading_from_pixels() {
        let strip = solid_strip([220, 40, 40]);
        for side in ["p1", "p2"] {
            let columns: Vec<u8> = super::super::decode::classify_columns(
                &strip,
                1920,
                1080,
                side,
                HUD_STRIP_Y as usize,
            )
            .into_iter()
            .map(|color| color as u8)
            .collect();

            let from_columns = hp_fill_ratio_from_columns(&columns, side);
            let from_pixels = hp_fill_ratio_with_quality_from_hud_strip(&strip, 1920, 1080, side);

            assert_eq!(from_columns, from_pixels, "{side} で答えが違う");
        }
    }

    /// 走査する行は上下を除いた内側だけ。ここがずれると枠の白を数えて
    /// しまう。
    #[test]
    fn the_scan_skips_the_bar_edges() {
        let scan = hp_column_scan("p1");

        assert_eq!(scan[3], HP_COL_ROW_SKIP_TOP as u32);
        assert_eq!(scan[4], (95 - 64) - HP_COL_ROW_SKIP_BOTTOM as u32);
        assert_eq!(scan[5], 1, "p1 は右下がり");
        assert_eq!(hp_column_scan("p2")[5], 0);
    }

    /// 左右で別の範囲を返す。取り違えると相手のバーを自分のスコアとして
    /// 数える。
    #[test]
    fn each_side_gets_its_own_range() {
        let (p1_x1, _, p1_x2, _) = hp_score_roi_in_strip("p1");
        let (p2_x1, _, p2_x2, _) = hp_score_roi_in_strip("p2");

        assert_eq!((p1_x1, p1_x2), (HP_ROI_P1.0, HP_ROI_P1.1));
        assert_eq!((p2_x1, p2_x2), (HP_ROI_P2.0, HP_ROI_P2.1));
        assert!(p1_x2 <= p2_x1, "左右の範囲が重なっている");
    }
}
