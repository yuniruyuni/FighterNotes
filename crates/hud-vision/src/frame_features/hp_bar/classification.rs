use super::*;

/// 1 画素がどの色に当たるか。
///
/// 優先順位は 純白 > 残量 > 残像 > 黄白 > 橙 > 空き。上から順に見て、
/// 最初に当たったところで決める。空きは「他のどれでもない」であって、
/// 空きらしい色があるわけではない。半透明の帯からステージ背景が透ける
/// ため、暗さでは判定できない。
pub(crate) fn classify_hp_pixel(r: f32, g: f32, b: f32, hue: HpFillHue) -> HpColColor {
    // 1. 純白。バーの枠。
    if r > 180.0 && g > 180.0 && b > 180.0 {
        return HpColColor::White;
    }

    let [h, s, v] = rgb_to_hsv(r, g, b);

    // 2. HP 充填色（通常時: 赤系 P1 / 青系 P2、ピンチ時: 黄は両サイド共通）
    // 低HP黄: g > r * 0.80 で純黄（H≥25相当）のみ Fill に分類。
    // H=22-27 の高輝度オレンジ（G/R < 0.80）はここを素通りして Orange に落とす。
    // ピンチ黄バーには微小な B グラデーション（実測 B=96〜113）があるため、
    // B 閾値に依存しない HSV 条件で判定する（YW の b>100 に任せると分裂する）。
    let is_primary = match hue {
        HpFillHue::Red => (h <= 20.0 || h >= 145.0) && s > 100.0 && v > 60.0,
        HpFillHue::Blue => {
            (88.0..=160.0).contains(&h) && strictly_above(s, 45.0) && strictly_above(v, 60.0)
        }
    };
    let is_pinch_yellow = (22.0..=35.0).contains(&h)
        && strictly_above(s, 120.0)
        && strictly_above(v, 200.0)
        && strictly_above(g, r * 0.80);
    if is_primary || is_pinch_yellow {
        return HpColColor::Fill;
    }

    // 3. ダメージゴースト（両サイド共通）: コンボで失った HP の暗い残像。
    // 実測値 P1: R≈137 G≈122 B≈39 / P2: R≈134 G≈118 B≈37（V≈135, S≈185）。
    // 明るい fill（V>200）とは V で明確に分離される。
    // G/R > 0.82 で暗い純橙（G/R<0.82）を除外する。
    // HP=0 の KO 直後はバー全域がこの色で点灯し続けるため、
    // Fill と区別しないと fill_ratio を誤読する。
    if (20.0..=30.0).contains(&h)
        && strictly_above(s, 150.0)
        && (100.0..200.0).contains(&v)
        && strictly_above(g, r * 0.82)
    {
        return HpColColor::Ghost;
    }

    // 4. 黄白（damage zone 境界: 明るい黄-橙系, B>100 で飽和黄 HP バーを除外）
    if r > 165.0 && g > 150.0 && b > 100.0 {
        return HpColColor::YellowWhite;
    }

    // 5. Orange ダメージゾーン
    // V 上限なし: 高輝度オレンジ（V≥200, G/R<0.80）も捕捉する。
    // 黄色 HP fill は上記 Fill 条件（g > r*0.80）で先に除外済み。
    if (10.0..=27.0).contains(&h) && strictly_above(s, 60.0) && strictly_above(v, 80.0) {
        return HpColColor::Orange;
    }

    HpColColor::Dark
}

/// 列全体の採択閾値:
///   White ≥50%, Fill ≥10%, Ghost ≥40%, YellowWhite ≥40%, Orange ≥15%
pub(crate) fn classify_hp_col(roi: &SlantedRoi<'_>, column: usize, hue: HpFillHue) -> HpColColor {
    let row_start = HP_COL_ROW_SKIP_TOP.min(roi.height);
    let row_end = roi
        .height
        .saturating_sub(HP_COL_ROW_SKIP_BOTTOM)
        .max(row_start);

    let mut n_white = 0usize;
    let mut n_fill = 0usize;
    let mut n_ghost = 0usize;
    let mut n_yw = 0usize;
    let mut n_orange = 0usize;
    let mut total = 0usize;

    for ry in row_start..row_end {
        let Some([r, g, b]) = roi.rgb_at(column, ry, row_start) else {
            continue;
        };
        total += 1;
        match classify_hp_pixel(r, g, b, hue) {
            HpColColor::White => n_white += 1,
            HpColColor::Fill => n_fill += 1,
            HpColColor::Ghost => n_ghost += 1,
            HpColColor::YellowWhite => n_yw += 1,
            HpColColor::Orange => n_orange += 1,
            HpColColor::Dark => {}
        }
    }

    if total == 0 {
        return HpColColor::Dark;
    }
    let t = total as f32;
    if n_white as f32 / t >= 0.50 {
        return HpColColor::White;
    }
    // 低HP黄状態のふちどりは黄みがかった白になる（B channel が低下して純白判定を外れる）。
    // White+YW が支配的で Fill/Ghost/Orange がなければ White として扱う。
    if n_white as f32 / t >= 0.10
        && (n_white + n_yw) as f32 / t >= 0.80
        && n_fill == 0
        && n_ghost == 0
        && n_orange == 0
    {
        return HpColColor::White;
    }
    if n_fill as f32 / t >= 0.10 {
        return HpColColor::Fill;
    }
    if n_ghost as f32 / t >= 0.40 {
        return HpColColor::Ghost;
    }
    if n_yw as f32 / t >= 0.40 {
        return HpColColor::YellowWhite;
    }
    if n_orange as f32 / t >= 0.15 {
        return HpColColor::Orange;
    }
    HpColColor::Dark
}

/// 列色配列を同色の連続区間（ゾーン）に圧縮する。
pub(crate) fn segment_zones(col_colors: &[HpColColor]) -> Vec<HpZone> {
    let mut zones: Vec<HpZone> = Vec::new();
    if col_colors.is_empty() {
        return zones;
    }
    let mut cur = col_colors[0];
    let mut start = 0usize;
    for (cy, &c) in col_colors.iter().enumerate().skip(1) {
        if c != cur {
            zones.push(HpZone {
                color: cur,
                start,
                end: cy - 1,
            });
            cur = c;
            start = cy;
        }
    }
    zones.push(HpZone {
        color: cur,
        start,
        end: col_colors.len() - 1,
    });
    zones
}

#[cfg(test)]
mod tests;
