//! viewer / examples 用のデバッグ JSON API
//!
//! frame_features.rs からの機械的分割（挙動不変）。

use super::*;

// ─────────────────────────────────────────────────────────────────────────────
// デバッグ用 pub API
// ─────────────────────────────────────────────────────────────────────────────

/// HP バーデコードの詳細情報を JSON 文字列で返す（examples/debug_hp_bar 用）。
///
/// 各列の色分類・ゾーン・ステートマシン結果を含む。
/// 全フレーム RGBA バッファを受け取る（y_strip_start=0 固定）。
pub fn hp_bar_debug_json(rgba: &[u8], width: u32, height: u32, side: &str) -> String {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    let x1 = x1u as usize;
    let x2 = x2u as usize;
    let y1 = y1u as usize;
    let roi_w = x2 - x1;
    let roi_h = y2u as usize - y1;
    let slope = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };
    let hue = if side == "p1" {
        HpFillHue::Red
    } else {
        HpFillHue::Blue
    };
    let y_strip_start = 0usize;
    let roi = SlantedRoi {
        rgba,
        frame_width: width as usize,
        x: std::ops::Range { start: x1, end: x2 },
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    // 表示は画面座標順（アンカー正規化は hp_bar_decode 内部のみ）
    let col_colors: Vec<HpColColor> = (0..roi_w)
        .map(|column| classify_hp_col(&roi, column, hue))
        .collect();

    let zones = segment_zones(&col_colors);
    let decode = hp_bar_decode(rgba, width, height, side, y_strip_start);

    fn col_char(c: HpColColor) -> char {
        match c {
            HpColColor::White => 'W',
            HpColColor::Fill => 'F',
            HpColColor::Ghost => 'G',
            HpColColor::YellowWhite => 'Y',
            HpColColor::Orange => 'O',
            HpColColor::Dark => 'D',
        }
    }
    fn col_name(c: HpColColor) -> &'static str {
        match c {
            HpColColor::White => "White",
            HpColColor::Fill => "Fill",
            HpColColor::Ghost => "Ghost",
            HpColColor::YellowWhite => "YW",
            HpColColor::Orange => "Orange",
            HpColColor::Dark => "Dark",
        }
    }

    let col_str: String = col_colors.iter().map(|&c| col_char(c)).collect();

    let zones_json: Vec<String> = zones
        .iter()
        .map(|z| {
            format!(
                r#"{{"c":"{}","s":{},"e":{},"w":{}}}"#,
                col_name(z.color),
                z.start,
                z.end,
                z.width()
            )
        })
        .collect();

    let fe = decode
        .fill_edge_cy
        .map_or("null".to_string(), |v| v.to_string());
    let dl = decode
        .damage_left_cy
        .map_or("null".to_string(), |v| v.to_string());

    format!(
        r#"{{"fill_ratio":{:.4},"orange_fill":{:.4},"uncertain":{},"fill_edge_cy":{},"damage_left_cy":{},"roi":{{"x1":{},"x2":{},"y1":{},"y2":{}}},"zones":[{}],"cols":"{}"}}"#,
        decode.fill_ratio,
        decode.orange_fill,
        decode.uncertain,
        fe,
        dl,
        x1,
        x2,
        y1,
        y2u,
        zones_json.join(","),
        col_str,
    )
}

/// 指定列範囲の per-row RGBA・HSV・色分類を JSON 配列で返す（examples/debug_hp_bar 用）。
pub fn hp_col_pixel_detail_json(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    cy_from: usize,
    cy_to: usize,
) -> String {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    let x1 = x1u as usize;
    let x2 = x2u as usize;
    let y1 = y1u as usize;
    let roi_h = y2u as usize - y1;
    let slope = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };
    let hue = if side == "p1" {
        HpFillHue::Red
    } else {
        HpFillHue::Blue
    };
    let y_strip_start = 0usize;
    let roi = SlantedRoi {
        rgba,
        frame_width: width as usize,
        x: std::ops::Range { start: x1, end: x2 },
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
    let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);

    let mut cols_json: Vec<String> = Vec::new();
    for cy in cy_from..=cy_to.min(x2 - x1 - 1) {
        let col_color = classify_hp_col(&roi, cy, hue);
        let col_name = match col_color {
            HpColColor::White => "White",
            HpColColor::Fill => "Fill",
            HpColColor::Ghost => "Ghost",
            HpColColor::YellowWhite => "YW",
            HpColColor::Orange => "Orange",
            HpColColor::Dark => "Dark",
        };

        let mut rows_json: Vec<String> = Vec::new();
        let mut n_w = 0usize;
        let mut n_f = 0usize;
        let mut n_y = 0usize;
        let mut n_o = 0usize;
        let mut total = 0usize;

        for ry in row_start..row_end {
            // 座標も画素も、解析が使う ROI に訊く。ここで別に計算すると、
            // 表示が本体とは違う場所を指しうる。
            if let (Some(gx), Some([r, g, b])) = (
                roi.column_x(cy, ry, row_start),
                roi.rgb_at(cy, ry, row_start),
            ) {
                total += 1;
                let [h, s, v] = rgb_to_hsv(r, g, b);

                // 表示は解析と同じ判定を使う。ここで別に書くと、表示が
                // 正しく見えるのに解析は違うものを読んでいる状態になる。
                let px_class = match classify_hp_pixel(r, g, b, hue) {
                    HpColColor::White => {
                        n_w += 1;
                        "W"
                    }
                    HpColColor::Fill => {
                        n_f += 1;
                        "F"
                    }
                    HpColColor::Ghost => "G",
                    HpColColor::YellowWhite => {
                        n_y += 1;
                        "Y"
                    }
                    HpColColor::Orange => {
                        n_o += 1;
                        "O"
                    }
                    HpColColor::Dark => "D",
                };

                rows_json.push(format!(
                    r#"{{"ry":{},"gx":{},"r":{},"g":{},"b":{},"h":{:.0},"s":{:.0},"v":{:.0},"cls":"{}"}}"#,
                    ry, gx, r as u8, g as u8, b as u8, h, s, v, px_class
                ));
            }
        }

        cols_json.push(format!(
            r#"{{"cy":{},"col_cls":"{}","total":{},"nW":{},"nF":{},"nY":{},"nO":{},"rows":[{}]}}"#,
            cy,
            col_name,
            total,
            n_w,
            n_f,
            n_y,
            n_o,
            rows_json.join(",")
        ));
    }

    format!("[{}]", cols_json.join(","))
}

// ─────────────────────────────────────────────────────────────────────────────
// テスト
