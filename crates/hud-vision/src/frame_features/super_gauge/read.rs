//! ラベルとバーの読みを一つの値にまとめる。
//!
//! 整数部はラベル、少数部はバー。ラベルが読めなければ全体を確定させない。
//! バーだけでは今が何ストック目かが判らないため。

use super::bar::read_fraction;
use super::label::{classify_digit, digit_component, looks_like_ca, white_components};
use super::model::SuperGaugeRead;
use super::pixels::{patch_fits, Patch};

pub(super) fn read_gauge(
    rgba: &[u8],
    frame_width: usize,
    label: Patch,
    bar: Patch,
    is_left: bool,
) -> SuperGaugeRead {
    if !patch_fits(rgba, frame_width, label) || !patch_fits(rgba, frame_width, bar) {
        return SuperGaugeRead::default();
    }

    let components = white_components(rgba, frame_width, label);
    let critical_art = looks_like_ca(rgba, frame_width, label, &components);
    let displayed_level = if critical_art {
        Some(3)
    } else {
        digit_component(&components, label.width, is_left)
            .and_then(|component| classify_digit(rgba, frame_width, label, component))
    };
    let fraction = read_fraction(rgba, frame_width, bar, is_left);
    let value = displayed_level.map_or(fraction, |level| {
        if level >= 3 {
            3.0
        } else {
            // 次ストック獲得直前でも表示整数部はまだ変わっていない。
            // ちょうど N.000 に丸めると時間補正層が整数ラベルを誤るため、
            // 少数部は 1.0 未満に保つ。
            level as f32 + fraction.min(0.995)
        }
    });

    SuperGaugeRead {
        value,
        displayed_level,
        critical_art,
        uncertain: displayed_level.is_none(),
    }
}
