//! SA ゲージの整数ラベルを読む。
//!
//! ラベルは白抜きの一文字（0〜3）か、CA 表示の二文字。どちらも白画素の
//! かたまりとして拾い、形で見分ける。ステージの明るい部分もかたまりに
//! 見えるので、大きさと位置で絞ってから形を調べる。

use std::collections::VecDeque;

use super::pixels::{is_glyph_white, neighbors, rgb_at, Patch};

/// ラベル内で見つけた白のかたまり。座標はパッチ内の相対位置。
#[derive(Debug, Clone, Copy)]
pub(super) struct WhiteComponent {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    area: usize,
    /// 走査で最初に触れた画素。かたまり本体に属することが保証される。
    seed_x: usize,
    seed_y: usize,
}

impl WhiteComponent {
    fn width(self) -> usize {
        self.x1 - self.x0 + 1
    }

    fn height(self) -> usize {
        self.y1 - self.y0 + 1
    }

    fn center_x(self) -> usize {
        (self.x0 + self.x1) / 2
    }

    /// 文字らしい大きさか。細かい点や背の低い破片は、ステージの明るい
    /// 部分がラベルに掛かっただけ。文字として扱うと、数字の位置に
    /// 一番近いという理由でそれが選ばれる。
    fn looks_like_a_glyph(self, patch_height: usize) -> bool {
        /// これ未満の画素数は文字ではない。
        const MIN_AREA: usize = 45;
        /// ラベルの高さのうち、文字が占める最小の割合。
        const MIN_HEIGHT_NUMERATOR: usize = 2;
        const MIN_HEIGHT_DENOMINATOR: usize = 5;
        self.area >= MIN_AREA
            && self.height() >= patch_height * MIN_HEIGHT_NUMERATOR / MIN_HEIGHT_DENOMINATOR
    }
}

/// 数字が出るはずの位置。ラベルの中で左右対称に置かれる。
///
/// ここがずれると、数字ではなくステージの明るい破片の方が「近い」ことに
/// なって選ばれる。
fn expected_digit_centre(label_width: usize, is_left: bool) -> usize {
    /// 位置を決めるときの基準にするラベル幅。
    const REFERENCE_WIDTH: usize = 90;
    /// 基準幅の中での位置。左側のゲージは右寄り、右側は左寄り。
    const CENTRE_ON_THE_LEFT: usize = 72;
    const CENTRE_ON_THE_RIGHT: usize = 26;

    let at = if is_left {
        CENTRE_ON_THE_LEFT
    } else {
        CENTRE_ON_THE_RIGHT
    };
    label_width * at / REFERENCE_WIDTH
}

/// ラベル内の白のかたまりを拾う。小さすぎるものと背の低いものは
/// 文字ではないので落とす。
pub(super) fn white_components(
    rgba: &[u8],
    frame_width: usize,
    patch: Patch,
) -> Vec<WhiteComponent> {
    let len = patch.width * patch.height;
    let mut white = vec![false; len];
    for y in 0..patch.height {
        for x in 0..patch.width {
            white[y * patch.width + x] =
                is_glyph_white(rgb_at(rgba, frame_width, patch.x + x, patch.y + y));
        }
    }

    let mut seen = vec![false; len];
    let mut components = Vec::new();
    for seed in 0..len {
        if seen[seed] || !white[seed] {
            continue;
        }
        seen[seed] = true;
        let mut queue = VecDeque::from([seed]);
        let (mut x0, mut x1) = (patch.width, 0);
        let (mut y0, mut y1) = (patch.height, 0);
        let mut area = 0;
        while let Some(index) = queue.pop_front() {
            let x = index % patch.width;
            let y = index / patch.width;
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
            area += 1;
            for neighbor in neighbors(x, y, patch.width, patch.height) {
                if !seen[neighbor] && white[neighbor] {
                    seen[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        let component = WhiteComponent {
            x0,
            x1,
            y0,
            y1,
            area,
            seed_x: seed % patch.width,
            seed_y: seed / patch.width,
        };
        if component.looks_like_a_glyph(patch.height) {
            components.push(component);
        }
    }
    components
}

/// CA 表示かどうか。並んだ二文字で、右の A にだけ閉じた穴がある。
/// 数字とステージの明るい破片が二つのかたまりに見える場面と区別する。
pub(super) fn looks_like_ca(
    rgba: &[u8],
    frame_width: usize,
    patch: Patch,
    components: &[WhiteComponent],
) -> bool {
    let mut glyphs: Vec<_> = components
        .iter()
        .copied()
        .filter(|component| component.width() >= 8 && component.area >= 130)
        .collect();
    glyphs.sort_by_key(|component| component.x0);
    glyphs.windows(2).any(|pair| {
        let left = pair[0];
        let right = pair[1];
        right.x0 <= left.x1 + 18
            && right.x1 - left.x0 + 1 >= 45
            && left.y0.abs_diff(right.y0) <= 12
            && left.y1.abs_diff(right.y1) <= 12
            && !has_enclosed_hole(rgba, frame_width, patch, left)
            && has_enclosed_hole(rgba, frame_width, patch, right)
    })
}

/// 数字が出るはずの位置に最も近いかたまりを選ぶ。位置は左右で鏡像。
pub(super) fn digit_component(
    components: &[WhiteComponent],
    label_width: usize,
    is_left: bool,
) -> Option<WhiteComponent> {
    let expected_center = expected_digit_centre(label_width, is_left);
    components
        .iter()
        .copied()
        .filter(|component| component.width() <= label_width / 2)
        .min_by_key(|component| component.center_x().abs_diff(expected_center))
}

/// かたまりの形から数字を決める。判らなければ None を返し、読み取りを
/// 確定させない。
pub(super) fn classify_digit(
    rgba: &[u8],
    frame_width: usize,
    patch: Patch,
    component: WhiteComponent,
) -> Option<u8> {
    let width = component.width();
    let height = component.height();
    if height < patch.height * 3 / 5 {
        return None;
    }
    if width * 100 / height <= 38 {
        return Some(1);
    }
    // 0 と 3 は上下端の張り出しが似ており、3 の左上／左下を
    // 矩形の塗り率だけで見ると 0 に誤分類しやすい。0 にだけある
    // 閉じた内側領域を直接調べる。
    if has_enclosed_hole(rgba, frame_width, patch, component) {
        return Some(0);
    }

    let right_upper = region_fill(
        rgba,
        frame_width,
        patch,
        component,
        (0.62, 1.00, 0.27, 0.47),
    );
    let left_lower = region_fill(
        rgba,
        frame_width,
        patch,
        component,
        (0.00, 0.38, 0.58, 0.78),
    );
    let right_lower = region_fill(
        rgba,
        frame_width,
        patch,
        component,
        (0.62, 1.00, 0.58, 0.78),
    );

    if right_upper >= 0.12 && left_lower > right_lower + 0.08 {
        Some(2)
    } else if right_upper >= 0.12 && right_lower >= 0.12 {
        Some(3)
    } else {
        None
    }
}

/// かたまりの内側に、外へ通じていない領域があるか。0 と A を
/// 3 や C から見分ける手がかり。
fn has_enclosed_hole(
    rgba: &[u8],
    frame_width: usize,
    patch: Patch,
    component: WhiteComponent,
) -> bool {
    let width = component.width();
    let height = component.height();
    let mut near_white = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            near_white[y * width + x] = is_glyph_white(rgb_at(
                rgba,
                frame_width,
                patch.x + component.x0 + x,
                patch.y + component.y0 + y,
            ));
        }
    }

    // 外接矩形の中にはステージやエフェクトの白画素も入る。文字の seed と
    // 繋がっている部分だけに限定し、背景が穴を閉じたように見えるのを防ぐ。
    let seed_x = component.seed_x - component.x0;
    let seed_y = component.seed_y - component.y0;
    let seed = seed_y * width + seed_x;
    if !near_white[seed] {
        return false;
    }
    let mut white = vec![false; width * height];
    white[seed] = true;
    let mut component_queue = VecDeque::from([seed]);
    while let Some(index) = component_queue.pop_front() {
        let x = index % width;
        let y = index / width;
        for neighbor in neighbors(x, y, width, height) {
            if near_white[neighbor] && !white[neighbor] {
                white[neighbor] = true;
                component_queue.push_back(neighbor);
            }
        }
    }

    // アンチエイリアスで輪郭に生じる 1px 程度の隙間だけを閉じる。
    let original = white.clone();
    for y in 0..height {
        for x in 0..width {
            if original[y * width + x] {
                for neighbor_y in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                    for neighbor_x in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                        white[neighbor_y * width + neighbor_x] = true;
                    }
                }
            }
        }
    }

    let mut outside = vec![false; width * height];
    let mut queue = VecDeque::new();
    for x in 0..width {
        enqueue_background(&white, &mut outside, &mut queue, x, 0, width);
        enqueue_background(&white, &mut outside, &mut queue, x, height - 1, width);
    }
    for y in 0..height {
        enqueue_background(&white, &mut outside, &mut queue, 0, y, width);
        enqueue_background(&white, &mut outside, &mut queue, width - 1, y, width);
    }
    while let Some(index) = queue.pop_front() {
        let x = index % width;
        let y = index / width;
        for neighbor in neighbors(x, y, width, height) {
            if !white[neighbor] && !outside[neighbor] {
                outside[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }

    let enclosed = white
        .iter()
        .zip(outside.iter())
        .filter(|(white, outside)| !**white && !**outside)
        .count();
    enclosed * 40 >= width * height
}

fn enqueue_background(
    white: &[bool],
    outside: &mut [bool],
    queue: &mut VecDeque<usize>,
    x: usize,
    y: usize,
    width: usize,
) {
    let index = y * width + x;
    if !white[index] && !outside[index] {
        outside[index] = true;
        queue.push_back(index);
    }
}

/// かたまりの外接矩形を割合で切った区画の、白画素の占める割合。
fn region_fill(
    rgba: &[u8],
    frame_width: usize,
    patch: Patch,
    component: WhiteComponent,
    normalized: (f32, f32, f32, f32),
) -> f32 {
    let (nx0, nx1, ny0, ny1) = normalized;
    let x0 = component.x0 + (component.width() as f32 * nx0) as usize;
    let x1 = component.x0 + (component.width() as f32 * nx1).ceil() as usize;
    let y0 = component.y0 + (component.height() as f32 * ny0) as usize;
    let y1 = component.y0 + (component.height() as f32 * ny1).ceil() as usize;
    let mut white = 0usize;
    let mut total = 0usize;
    for y in y0..y1.min(patch.height) {
        for x in x0..x1.min(patch.width) {
            let pixel = rgb_at(rgba, frame_width, patch.x + x, patch.y + y);
            white += usize::from(is_glyph_white(pixel));
            total += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        white as f32 / total as f32
    }
}

#[cfg(test)]
mod tests;
