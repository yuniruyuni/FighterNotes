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
        x: x1..x2,
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
        x: x1..x2,
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    let w_px = width as usize;
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
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx_i = x1 as i32 + cy as i32 + x_off;
            // classify_hp_col と同じ境界チェック（ROI [x1, x2) でクリップ）
            if gx_i < x1 as i32 || gx_i as usize >= x2 {
                continue;
            }
            let gx = gx_i as usize;
            let idx = ((y1 + ry - y_strip_start) * w_px + gx) * 4;
            if idx + 2 >= rgba.len() {
                continue;
            }
            total += 1;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;
            let [h, s, v] = rgb_to_hsv(r, g, b);

            let px_class = if r > 180.0 && g > 180.0 && b > 180.0 {
                n_w += 1;
                "W"
            } else {
                let primary = match hue {
                    HpFillHue::Red => (h <= 20.0 || h >= 145.0) && s > 100.0 && v > 60.0,
                    HpFillHue::Blue => (88.0..=160.0).contains(&h) && s > 45.0 && v > 60.0,
                };
                let fill = primary
                    || ((22.0..=35.0).contains(&h) && s > 120.0 && v > 200.0 && g > r * 0.80);
                let ghost = (20.0..=30.0).contains(&h)
                    && s > 150.0
                    && (100.0..200.0).contains(&v)
                    && g > r * 0.82;
                if fill {
                    n_f += 1;
                    "F"
                } else if ghost {
                    "G"
                } else if r > 165.0 && g > 150.0 && b > 100.0 {
                    n_y += 1;
                    "Y"
                } else if (10.0..=27.0).contains(&h) && s > 60.0 && v > 80.0 {
                    n_o += 1;
                    "O"
                } else {
                    "D"
                }
            };

            rows_json.push(format!(
                r#"{{"ry":{},"gx":{},"r":{},"g":{},"b":{},"h":{:.0},"s":{:.0},"v":{:.0},"cls":"{}"}}"#,
                ry, gx,
                rgba[idx], rgba[idx+1], rgba[idx+2],
                h, s, v, px_class
            ));
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
