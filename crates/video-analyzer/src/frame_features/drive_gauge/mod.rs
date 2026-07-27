//! ドライブゲージ検出（列分類・アンカー正規化・ゾーンデコード・EMPTY 検出）
//!
//! frame_features.rs からの機械的分割（挙動不変）。

use super::*;

// -------------------------------------------------------------------------
// ドライブゲージ検出（列分類 → アンカー正規化 → ゾーンデコード）
// -------------------------------------------------------------------------
//
// HP バーと同じ 2-Step 構成:
//   Step 1: 斜め列を色分類し、アンカー起点（index 0 = 画面中央側）に正規化
//   Step 2: サイド非依存の decode_drive_zones でゾーン文法を検証
//
// ゲージ構造（1920x1080 実測）:
//   - 6 セル（ピッチ ≈54px、セル間ギャップ 2〜4px）の平行四辺形、y=114〜131
//   - 残量は中央側に寄り、外側から減る（アンカー = 中央端）
//   - 点灯色: 黄→緑グラデーション H=30〜51 + 低残量橙警告 H=20（すべて S>200, V>140）
//   - バーンアウト: ゲージ全体が暗転し、灰白の回復バー（S<20, V≈150-165）が
//     中央側から外側へ成長。全幅到達で通常表示に復帰

/// ドライブゲージ ROI（1920x1080 基準、上端行の座標。行ごとに slope だけずれる）
pub(crate) const DRIVE_ROI_LEFT: (u32, u32, u32, u32) = (561, 885, 114, 132);
pub(crate) const DRIVE_ROI_RIGHT: (u32, u32, u32, u32) = (1036, 1360, 114, 132);
/// 斜め列の 1 行あたり x ずれ（左ゲージ +、右ゲージ −。実測 10px/16行）
pub(crate) const DRIVE_BAR_SLOPE: f32 = 0.625;

mod classification;
mod debug;
mod decode;
mod model;
mod read;

#[cfg(test)]
pub(crate) use classification::segment_drive_runs;
pub use debug::drive_bar_debug_json;
#[cfg(test)]
pub(crate) use decode::decode_drive_runs;
#[cfg(test)]
pub(crate) use model::DriveColClass;
pub use model::DriveGaugeRead;

/// RGBA バッファからドライブゲージを読み取る。side は "left" / "right"。
pub fn drive_gauge_read(rgba: &[u8], width: u32, height: u32, side: &str) -> DriveGaugeRead {
    read::drive_gauge_read_impl(rgba, width, height, side, 0)
}

pub fn drive_gauge_read_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> DriveGaugeRead {
    let y0 = (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize;
    read::drive_gauge_read_impl(strip, full_width, full_height, side, y0)
}

/// 旧 API 互換: ドライブゲージ充填率（0.0–1.0 = value/6）。
/// バーンアウト中は 0.0 を返す。
pub fn drive_fill_ratio(rgba: &[u8], width: u32, height: u32, side: &str) -> f32 {
    let d = drive_gauge_read(rgba, width, height, side);
    if d.burnout {
        0.0
    } else {
        d.value / 6.0
    }
}

pub fn drive_fill_ratio_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> f32 {
    let d = drive_gauge_read_from_hud_strip(strip, full_width, full_height, side);
    if d.burnout {
        0.0
    } else {
        d.value / 6.0
    }
}
