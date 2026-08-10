use super::*;

mod state_machine;

pub(crate) use state_machine::decode_hp_zones;

/// HP バー全列を分類→アンカー正規化→ゾーン分割→ステートマシンで構造を検証し、
/// 充填端・ダメージ端を検出する。
///
/// アルゴリズム:
///   1. 各斜め列を色分類し、アンカー起点（index 0 = cap 端）の配列に正規化
///      （P1 はアンカーが画面右端なので cy 逆順、P2 は左端なのでそのまま）
///   2. ゾーン分割後、サイド非依存の decode_hp_zones で前方スキャン
///   3. 結果のアンカー相対 index を画面 cy に逆変換して返す
pub(crate) fn hp_bar_decode(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> HpBarDecode {
    let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
    // 潰れた ROI はゾーンが一つも取れず、そのまま「読めなかった」に落ちる。
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    let x1 = x1u as usize;
    let x2 = x2u as usize;
    let y1 = y1u as usize;
    let roi_w = x2 - x1;
    let roi_h = y2u as usize - y1;
    let slope: f32 = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };
    let is_p1 = side == "p1";
    let roi = SlantedRoi {
        rgba,
        frame_width: width as usize,
        x: x1..x2,
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    // 全列をアンカー起点（index 0 = cap 端）で分類。
    // P1 はアンカーが画面右端なので cy を逆順に、P2 は左端なのでそのまま読む。
    // 以降の処理はサイド非依存（fill 色の差は classify_hp_col の hue で吸収済み）。
    let hue = if is_p1 {
        HpFillHue::Red
    } else {
        HpFillHue::Blue
    };
    let classify = |column: usize| classify_hp_col(&roi, column, hue);
    let col_colors: Vec<HpColColor> = if is_p1 {
        (0..roi_w).rev().map(classify).collect()
    } else {
        (0..roi_w).map(classify).collect()
    };

    let zones = segment_zones(&col_colors);
    let d = decode_hp_zones(&zones, roi_w);

    // アンカー相対 index → 画面 cy 逆変換（デバッグ出力互換）
    let to_cy = |a: usize| if is_p1 { roi_w - 1 - a } else { a };
    HpBarDecode {
        fill_ratio: d.fill_ratio,
        orange_fill: d.orange_fill,
        uncertain: d.uncertain,
        fill_edge_cy: d.fill_edge_a.map(to_cy),
        damage_left_cy: d.damage_left_a.map(to_cy),
    }
}

pub(crate) fn hp_fill_ratio_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> (f32, bool) {
    // classify_hp_col → segment_zones → hp_bar_decode ステートマシンで一本化。
    // fill_ratio / uncertain ともに decode から取得する。
    let decode = hp_bar_decode(rgba, width, height, side, y_strip_start);
    (decode.fill_ratio, decode.uncertain)
}
