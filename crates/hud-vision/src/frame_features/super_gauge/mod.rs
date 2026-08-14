//! 画面下端の SA ゲージ読み取り。
//!
//! 整数部は固定位置の 0〜3 / CA グリフ、少数部は左右対称の色付きバーから読む。
//! browser では必要パッチだけを既存 HUD strip の未使用領域へ詰めて転送する。

use super::*;

mod bar;
mod debug;
mod label;
mod model;
mod pixels;
mod read;

pub use debug::super_gauge_debug_json;
pub use model::SuperGaugeRead;
use pixels::Patch;
use read::read_gauge;

/// 整数ラベルが下がっても、これ未満の差はストック境界の表示揺れであり、
/// SA 消費として扱わない。
pub const MIN_SUPER_SPEND_DROP: f32 = 0.65;

const FULL_LABEL_LEFT: Patch = Patch {
    x: 55,
    y: 955,
    width: 90,
    height: 75,
};
const FULL_BAR_LEFT: Patch = Patch {
    x: 145,
    y: 975,
    width: 265,
    height: 50,
};
const FULL_BAR_RIGHT: Patch = Patch {
    x: 1510,
    y: 975,
    width: 265,
    height: 50,
};
const FULL_LABEL_RIGHT: Patch = Patch {
    x: 1775,
    y: 955,
    width: 90,
    height: 75,
};

/// 等倍で置いた帯の中での位置。縮小していないので、フレームと同じ大きさ。
pub const SUPER_STRIP_H: u32 = 75;
pub(crate) const NATIVE_LABEL_LEFT: (usize, usize, usize, usize) = (0, 0, 90, 75);
pub(crate) const NATIVE_BAR_LEFT: (usize, usize, usize, usize) = (100, 0, 265, 50);
pub(crate) const NATIVE_BAR_RIGHT: (usize, usize, usize, usize) = (1555, 0, 265, 50);
pub(crate) const NATIVE_LABEL_RIGHT: (usize, usize, usize, usize) = (1830, 0, 90, 75);

pub(crate) const PACKED_LABEL_LEFT: (usize, usize, usize, usize) = (0, 0, 90, 70);
pub(crate) const PACKED_BAR_LEFT: (usize, usize, usize, usize) = (100, 32, 265, 38);
pub(crate) const PACKED_BAR_RIGHT: (usize, usize, usize, usize) = (1555, 32, 265, 38);
pub(crate) const PACKED_LABEL_RIGHT: (usize, usize, usize, usize) = (1830, 0, 90, 70);

/// 1920x1080 RGBA フレームから SA ゲージを読む。side は "left" / "right"。
pub fn super_gauge_read(rgba: &[u8], width: u32, height: u32, side: &str) -> SuperGaugeRead {
    if width != 1920 || height != 1080 {
        return SuperGaugeRead::default();
    }
    if side == "left" {
        read_gauge(rgba, width as usize, FULL_LABEL_LEFT, FULL_BAR_LEFT, true)
    } else {
        read_gauge(
            rgba,
            width as usize,
            FULL_LABEL_RIGHT,
            FULL_BAR_RIGHT,
            false,
        )
    }
}

/// browser が HUD strip に埋め込んだ SA パッチを読む。
pub fn super_gauge_read_from_hud_strip(
    strip: &[u8],
    full_width: u32,
    side: &str,
) -> SuperGaugeRead {
    let patch = |(x, y, width, height)| Patch {
        x,
        y,
        width,
        height,
    };
    if full_width != 1920 {
        return SuperGaugeRead::default();
    }
    if side == "left" {
        read_gauge(
            strip,
            full_width as usize,
            patch(PACKED_LABEL_LEFT),
            patch(PACKED_BAR_LEFT),
            true,
        )
    } else {
        read_gauge(
            strip,
            full_width as usize,
            patch(PACKED_LABEL_RIGHT),
            patch(PACKED_BAR_RIGHT),
            false,
        )
    }
}

/// 等倍で置いた帯から SA ゲージを読む。
///
/// 縮小した strip から読む経路と違い、画素を落としていない。読み取りの判定は
/// どれも比率で書かれているので、行数が増えてもそのまま成り立つ。
pub fn super_gauge_read_from_native_strip(
    strip: &[u8],
    full_width: u32,
    side: &str,
) -> SuperGaugeRead {
    let patch = |(x, y, width, height)| Patch {
        x,
        y,
        width,
        height,
    };
    if full_width != 1920 {
        return SuperGaugeRead::default();
    }
    if side == "left" {
        read_gauge(
            strip,
            full_width as usize,
            patch(NATIVE_LABEL_LEFT),
            patch(NATIVE_BAR_LEFT),
            true,
        )
    } else {
        read_gauge(
            strip,
            full_width as usize,
            patch(NATIVE_LABEL_RIGHT),
            patch(NATIVE_BAR_RIGHT),
            false,
        )
    }
}

#[cfg(test)]
mod tests;
