use super::*;

mod chaining;
mod colored_badges;
mod column_colors;
mod monochrome;

use chaining::chain_badges;
use colored_badges::collect_colored_badges;
use column_colors::classify_color_columns;
use monochrome::detect_monochrome_controls;

/// バッジ帯をスキャンして色付き円・文字付き色箱・AUTO 箱・投げ円を検出する。
/// 戻り値: (badges, auto, throw)
pub(super) fn read_badges(
    f: &Frame,
    x_range: (u32, u32),
    mono_range: (u32, u32),
    is_p1: bool,
    y0: usize,
) -> (Vec<BadgeMark>, bool, bool) {
    let (x1, x2) = (x_range.0 as usize, x_range.1 as usize);
    let col_class = classify_color_columns(f, x1, x2, y0);
    let badge_spans = collect_colored_badges(f, x1, y0, &col_class);
    let badges = chain_badges(&badge_spans, x2 - x1, is_p1);
    let (auto, throw) =
        detect_monochrome_controls(f, mono_range, x1, x2 - x1, &col_class, &badges, y0);
    (badges, auto, throw)
}
