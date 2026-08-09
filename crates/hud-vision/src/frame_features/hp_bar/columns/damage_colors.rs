use super::palette::{is_damage_orange, is_low_health_yellow, ratio};
use super::scan::ColumnScan;
use crate::frame_features::HUD_STRIP_Y;

/// HP ROI 内の各列がオレンジ色（ダメージ受け中）かどうかを返す。
///
/// SF6 のダメージ表現: ダメージを受けた瞬間その部分がオレンジ色になり、
/// 一定時間後に透明になる。
pub fn hp_col_orange(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    hp_col_orange_impl(rgba, width, height, side, 0)
}

pub fn hp_col_orange_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> Vec<bool> {
    hp_col_orange_impl(
        strip,
        full_width,
        full_height,
        side,
        strip_start(full_height),
    )
}

fn hp_col_orange_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> Vec<bool> {
    match ColumnScan::new(width, height, side, y_strip_start) {
        Some(scan) => scan.columns_where(rgba, ratio::DAMAGE_ORANGE, is_damage_orange),
        None => Vec::new(),
    }
}

/// HP ROI 内の各列が黄色（低 HP: 残量 25% 以下）かどうかを返す。
///
/// SF6: HP が 25% 以下になるとバーが赤/青から黄色に変化する。
pub fn hp_col_yellow(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    hp_col_yellow_impl(rgba, width, height, side, 0)
}

pub fn hp_col_yellow_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    full_height: u32,
    side: &str,
) -> Vec<bool> {
    hp_col_yellow_impl(
        strip,
        full_width,
        full_height,
        side,
        strip_start(full_height),
    )
}

fn hp_col_yellow_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> Vec<bool> {
    match ColumnScan::new(width, height, side, y_strip_start) {
        Some(scan) => scan.columns_where(rgba, ratio::LOW_HEALTH_YELLOW, is_low_health_yellow),
        None => Vec::new(),
    }
}

/// HUD の帯だけを渡されたときの、帯の先頭行が画面の何行目か。
fn strip_start(full_height: u32) -> usize {
    (HUD_STRIP_Y as f32 * full_height as f32 / 1080.0) as usize
}
