//! SA ゲージのパッチと、その中の画素を見るための道具。
//!
//! ラベル側とバー側で同じ見方を使う。白の閾値や近傍の取り方が二つに
//! 分かれていると、片方だけ直したときに読みが食い違う。

/// フレーム上の矩形。browser は必要なパッチだけを HUD 帯へ詰めて渡すので、
/// 位置は渡された絵の中での座標になる。
#[derive(Clone, Copy)]
pub(super) struct Patch {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) width: usize,
    pub(super) height: usize,
}

/// パッチが絵の中に収まっているか。潰れた寸法や切り詰められたバッファで
/// 範囲外を読まないための門。
pub(super) fn patch_fits(rgba: &[u8], frame_width: usize, patch: Patch) -> bool {
    if frame_width == 0 || patch.width == 0 || patch.height == 0 {
        return false;
    }
    let frame_height = rgba.len() / 4 / frame_width;
    patch.x + patch.width <= frame_width && patch.y + patch.height <= frame_height
}

/// 絵の上の 1 画素の RGB。`patch_fits` を通した範囲でだけ呼ぶ。
pub(super) fn rgb_at(rgba: &[u8], frame_width: usize, x: usize, y: usize) -> [u8; 3] {
    let index = (y * frame_width + x) * 4;
    [rgba[index], rgba[index + 1], rgba[index + 2]]
}

/// グリフの塗りとみなす明るさか。数字も CA も白抜きで描かれる。
pub(super) fn is_glyph_white([r, g, b]: [u8; 3]) -> bool {
    r >= 190 && g >= 190 && b >= 190
}

/// 上下左右の隣。格子の外へは出ない。
pub(super) fn neighbors(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = usize> {
    let mut result = [None; 4];
    if x > 0 {
        result[0] = Some(y * width + x - 1);
    }
    if x + 1 < width {
        result[1] = Some(y * width + x + 1);
    }
    if y > 0 {
        result[2] = Some((y - 1) * width + x);
    }
    if y + 1 < height {
        result[3] = Some((y + 1) * width + x);
    }
    result.into_iter().flatten()
}

#[cfg(test)]
mod tests;
