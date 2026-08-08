use std::collections::VecDeque;

use super::{model::SuperGaugeRead, rgb_to_hsv};

#[derive(Clone, Copy)]
pub(super) struct Patch {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) width: usize,
    pub(super) height: usize,
}

#[derive(Debug, Clone, Copy)]
struct WhiteComponent {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    area: usize,
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
}

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

fn patch_fits(rgba: &[u8], frame_width: usize, patch: Patch) -> bool {
    if frame_width == 0 || patch.width == 0 || patch.height == 0 {
        return false;
    }
    let frame_height = rgba.len() / 4 / frame_width;
    patch.x + patch.width <= frame_width && patch.y + patch.height <= frame_height
}

fn white_components(rgba: &[u8], frame_width: usize, patch: Patch) -> Vec<WhiteComponent> {
    let len = patch.width * patch.height;
    let mut white = vec![false; len];
    for y in 0..patch.height {
        for x in 0..patch.width {
            let [r, g, b] = rgb_at(rgba, frame_width, patch.x + x, patch.y + y);
            white[y * patch.width + x] = r >= 190 && g >= 190 && b >= 190;
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
        if component.area >= 45 && component.height() >= patch.height * 2 / 5 {
            components.push(component);
        }
    }
    components
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> impl Iterator<Item = usize> {
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

fn looks_like_ca(
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
            // "CA" の右側 A には閉じた穴があり、左側 C にはない。
            // 数字と明るいステージ片が二成分に見えるケースを除外する。
            && !has_enclosed_hole(rgba, frame_width, patch, left)
            && has_enclosed_hole(rgba, frame_width, patch, right)
    })
}

fn digit_component(
    components: &[WhiteComponent],
    label_width: usize,
    is_left: bool,
) -> Option<WhiteComponent> {
    let expected_center = if is_left {
        label_width * 72 / 90
    } else {
        label_width * 26 / 90
    };
    components
        .iter()
        .copied()
        .filter(|component| component.width() <= label_width / 2)
        .min_by_key(|component| component.center_x().abs_diff(expected_center))
}

fn classify_digit(
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
            let [r, g, b] = rgb_at(
                rgba,
                frame_width,
                patch.x + component.x0 + x,
                patch.y + component.y0 + y,
            );
            near_white[y * width + x] = r >= 190 && g >= 190 && b >= 190;
        }
    }

    // bbox 内にはステージやエフェクトの白画素も入る。数字グリフの seed と
    // 連結している成分だけに限定し、背景が穴を閉じたように見えるのを防ぐ。
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
            let [r, g, b] = rgb_at(rgba, frame_width, patch.x + x, patch.y + y);
            white += usize::from(r >= 190 && g >= 190 && b >= 190);
            total += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        white as f32 / total as f32
    }
}

fn read_fraction(rgba: &[u8], frame_width: usize, patch: Patch, is_left: bool) -> f32 {
    let mut lit = Vec::with_capacity(patch.width);
    for screen_x in 0..patch.width {
        let mut colored = 0usize;
        for y in 0..patch.height {
            let [r, g, b] = rgb_at(rgba, frame_width, patch.x + screen_x, patch.y + y);
            let [h, s, v] = rgb_to_hsv(r as f32, g as f32, b as f32);
            let side_hue = if is_left {
                h >= 145.0 || h <= 7.0
            } else {
                (85.0..=130.0).contains(&h)
            };
            colored += usize::from(side_hue && s >= 90.0 && v >= 90.0);
        }
        lit.push(colored * 100 >= patch.height * 13);
    }
    if !is_left {
        lit.reverse();
    }

    let start_pad = (patch.width * 8 / 265).min(patch.width.saturating_sub(1));
    let end_pad = patch.width * 10 / 265;
    let usable_end = patch.width.saturating_sub(end_pad);
    if start_pad >= usable_end {
        return 0.0;
    }
    let usable = &lit[start_pad..usable_end];
    let Some(first) = usable.iter().position(|value| *value) else {
        return 0.0;
    };
    let max_edge_skip = (patch.width * 12 / 265).max(2);
    if first > max_edge_skip {
        return 0.0;
    }

    let max_gap = (patch.width * 5 / 265).max(2);
    let mut far = first;
    let mut gap = 0usize;
    for (index, value) in usable.iter().copied().enumerate().skip(first + 1) {
        if value {
            if gap > max_gap {
                break;
            }
            far = index;
            gap = 0;
        } else {
            gap += 1;
        }
    }
    ((far + 1) as f32 / usable.len() as f32).clamp(0.0, 1.0)
}

fn rgb_at(rgba: &[u8], frame_width: usize, x: usize, y: usize) -> [u8; 3] {
    let index = (y * frame_width + x) * 4;
    [rgba[index], rgba[index + 1], rgba[index + 2]]
}
