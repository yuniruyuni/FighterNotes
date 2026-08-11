//! SA ゲージのバーから、次ストックまでの溜まり具合を読む。
//!
//! バーは左右で色が違い、伸びる向きも逆。どちらもアンカー側から色の
//! ついた列が伸びていて、その遠端が現在値になる。
//!
//! 端の数列はバーの縁取りで、中身とは関係なく点く。途中の細かい途切れは
//! 半透明の背景が透けたもの。どちらも読みから外さないと、溜まりの量が
//! 実際とずれる。

use super::pixels::{rgb_at, Patch};
use super::rgb_to_hsv;

/// バーの色。左右で色相帯が違い、どちらも彩度と明度が高い。
/// 暗い背景や淡いエフェクトを溜まりと読まないための条件。
fn is_bar_colour(is_left: bool, [r, g, b]: [u8; 3]) -> bool {
    let [h, s, v] = rgb_to_hsv(r as f32, g as f32, b as f32);
    let side_hue = if is_left {
        // 左は赤。色相環をまたぐので、上端と下端の両方を拾う。
        h >= 145.0 || h <= 7.0
    } else {
        (85.0..=130.0).contains(&h)
    };
    side_hue && s >= 90.0 && v >= 90.0
}

/// 列ごとに、その列がバーの色で点いているかを返す。返す並びは
/// 常にアンカー側が先頭。右側のバーは逆向きに伸びるので反転する。
fn lit_columns(rgba: &[u8], frame_width: usize, patch: Patch, is_left: bool) -> Vec<bool> {
    let mut lit = Vec::with_capacity(patch.width);
    for column in 0..patch.width {
        let colored = (0..patch.height)
            .filter(|y| {
                is_bar_colour(
                    is_left,
                    rgb_at(rgba, frame_width, patch.x + column, patch.y + y),
                )
            })
            .count();
        // バーの上下は縁取りと影で暗い。列の一部が点いていれば足りる。
        lit.push(colored * 100 >= patch.height * 13);
    }
    if !is_left {
        lit.reverse();
    }
    lit
}

/// アンカー側から並べた点灯列から、溜まり具合を 0.0〜1.0 で出す。
pub(super) fn fraction_from_lit(lit: &[bool]) -> f32 {
    let columns = lit.len();
    // 両端の縁取りは中身と関係なく点くので、読む範囲から外す。
    let start_pad = (columns * 8 / 265).min(columns.saturating_sub(1));
    let end_pad = columns * 10 / 265;
    let usable_end = columns.saturating_sub(end_pad);
    let usable = lit.get(start_pad..usable_end).unwrap_or_default();

    let Some(first) = usable.iter().position(|value| *value) else {
        return 0.0;
    };
    // アンカーから離れたところで始まる光はバーではない。背景の色味を
    // 溜まりと読むと、空のゲージが満タンに見える。
    let max_edge_skip = (columns * 12 / 265).max(2);
    if first > max_edge_skip {
        return 0.0;
    }

    // 細かい途切れは背景の透け。繋いでその先まで数える。広い途切れの
    // 向こうは別物なので、そこで止める。
    let max_gap = (columns * 5 / 265).max(2);
    let mut far = first;
    let mut gap = 0usize;
    for (index, value) in usable.iter().copied().enumerate().skip(first) {
        if value {
            if gap > max_gap {
                return ((far + 1) as f32 / usable.len() as f32).clamp(0.0, 1.0);
            }
            far = index;
            gap = 0;
        } else {
            gap += 1;
        }
    }
    ((far + 1) as f32 / usable.len() as f32).clamp(0.0, 1.0)
}

/// バーのパッチから溜まり具合を読む。
pub(super) fn read_fraction(rgba: &[u8], frame_width: usize, patch: Patch, is_left: bool) -> f32 {
    fraction_from_lit(&lit_columns(rgba, frame_width, patch, is_left))
}

#[cfg(test)]
mod tests;
