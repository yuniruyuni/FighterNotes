use super::super::*;

/// HP ROI 内の各列が HP 色かどうかを返す（デバッグ・充填率計算の共通ヘルパー）。
///
/// 戻り値の長さ = ROI 幅（スケール済み列数）。空 ROI は空 Vec を返す。
pub fn hp_col_active(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    hp_col_active_impl(rgba, width, height, side, 0)
}

pub(crate) fn hp_col_active_impl(
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

    // 上下端を除いた中央行のみ使用（平行四辺形縁の AA ノイズ・"YOU" ラベルを回避）
    let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
    let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);

    // HP バーは平行四辺形。列 cx は row_start 基準の斜め列。
    // P1: 右下がり (+slope)、P2: 左下がり (-slope)。
    let slope: f32 = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };

    let mut col_active = vec![false; roi_w];
    for (cx, active) in col_active.iter_mut().enumerate() {
        let mut hp_count = 0usize; // 赤 or 青緑（通常 HP 色）
        let mut yellow_count = 0usize; // 黄色（低 HP ≤25%）
        let mut eff_count = 0usize; // ROI 内有効行数
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
            // SF6 HP バー色判定:
            // P1: 赤系（h≤20 または h≥145）で判定。低 HP 時は黄色（h=22-35）も別途検出。
            // P2: 青系（h=150-240 相当、OpenCV HSV では h=75-120 → ここでは 88-160）で判定。
            let is_hp_color = if side == "p1" {
                // キャラクタースプライトが HP ROI に重なる暗赤（s≈58〜78）を除外するため
                // s>100 に設定。HP バー本体 vivid crimson は s≈220 で確実に通過する。
                (h_hsv <= 20.0 || h_hsv >= 145.0) && s > 100.0 && v > 60.0
            } else {
                (88.0..=160.0).contains(&h_hsv) && s > 45.0 && v > 60.0
            };
            // HP 25% 以下では黄色（h=22-35, s>120, v>200）に変化
            // 閾値を 60% に上げることでキャラクター髪の毛などスプライトの黄色テクスチャを排除する
            // （HP バーは均一フラット色なので 60%+ の行が黄色になるが、髪はテクスチャで疎）
            let is_yellow = (22.0..=35.0).contains(&h_hsv) && s > 120.0 && v > 200.0;

            if is_hp_color {
                hp_count += 1;
            } else if is_yellow {
                yellow_count += 1;
            }
        }
        // P1: フレームメーターディザリング対応で閾値を 10% に下げる（22行中 3行以上で active）
        // P2: 15% 閾値を維持（遮蔽ノイズを許容）
        // 低 HP 黄色: 60% 閾値（髪の毛などスプライトが列の一部しか黄色でないのを除外）
        let hp_thresh = if side == "p1" { 0.10 } else { 0.15 };
        let active_by_hp = eff_count > 0 && (hp_count as f32 / eff_count as f32) > hp_thresh;
        let active_by_yellow = eff_count > 0 && (yellow_count as f32 / eff_count as f32) > 0.60;
        *active = active_by_hp || active_by_yellow;
    }
    col_active
}
