//! HP バー検出（ROI・列分類・ゾーン分割・ステートマシンデコード）
//!
//! frame_features.rs からの機械的分割（挙動不変）。

use super::*;

mod classification;
mod columns;
mod decode;
mod geometry;
mod model;
mod score;

pub(crate) use classification::{classify_hp_col, classify_hp_pixel, segment_zones};
pub use columns::*;
#[cfg(test)]
pub(crate) use decode::decode_hp_zones;
pub(crate) use decode::{hp_bar_decode, hp_fill_ratio_impl};
pub(crate) use geometry::hp_roi_base;
pub use geometry::{hp_parallelogram, HpParallelogram};
pub(crate) use model::*;
pub use score::*;

/// RGBA バッファから HP バー充填率（0.0–1.0）を返す。
///
/// Python 版 `hp_fill_ratio` の移植。
/// 列ごとに色相フィルタを適用し、HP 色の列の割合を返す。
/// 1920x1080 基準 ROI:
///   p1 (左): x=172–870,  y=64–95 → 赤系（hue>=145 || hue<=8, sat>55, val>70）
///   p2 (右): x=1050–1748, y=64–95 → 青緑系（hue 88–135, sat>45, val>60）
pub fn hp_fill_ratio(rgba: &[u8], width: u32, height: u32, side: &str) -> f32 {
    hp_fill_ratio_impl(rgba, width, height, side, 0).0
}

pub fn hp_fill_ratio_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> f32 {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_fill_ratio_impl(strip, full_width, full_height, side, y0).0
}

/// HP 充填率とアイランド有無（疑問フレーム判定）を返す。
pub fn hp_fill_ratio_with_quality(rgba: &[u8], width: u32, height: u32, side: &str) -> (f32, bool) {
    hp_fill_ratio_impl(rgba, width, height, side, 0)
}

pub fn hp_fill_ratio_with_quality_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> (f32, bool) {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_fill_ratio_impl(strip, full_width, full_height, side, y0)
}

/// damage zone の幅を ROI 幅で正規化して返す（境界ベース・安定版）。
///
/// `hp_col_orange` の単純列カウントと異なり、state machine で特定した
/// fill_edge と damage 境界の間の幅のみを採用するため雑音列に左右されない。
pub fn hp_damage_fill(rgba: &[u8], width: u32, height: u32, side: &str) -> f32 {
    hp_bar_decode(rgba, width, height, side, 0).orange_fill
}

pub fn hp_damage_fill_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> f32 {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    hp_bar_decode(strip, full_width, full_height, side, y0).orange_fill
}

/// HP バー列判定で使用する行の上下トリム量（平行四辺形縁・"YOU" ラベル等を除外）。
///
/// HP バー ROI 高さ 31px（y=64-95）のうち、上下の境界ピクセルを除外して
/// HP バー中央の確実な領域のみを使用する。
pub(crate) const HP_COL_ROW_SKIP_TOP: usize = 5; // 上から 5px 除外
pub(crate) const HP_COL_ROW_SKIP_BOTTOM: usize = 4; // 下から 4px 除外

/// SF6 HP バー（平行四辺形）の傾き（1920x1080 基準）。
///
/// 実測: 有効行 y=5→y=26（21 行差）で左端・右端とも 16px 右シフト。
/// P1 は +slope（右下）、P2 は −slope（左下）。スキャン列インデックス cx は
/// row_start 行での x 位置を基準とし、各行で `(ry - row_start) * slope` を加算する。
pub(crate) const HP_BAR_SLOPE: f32 = 0.75; // 3/4（整数比率）
