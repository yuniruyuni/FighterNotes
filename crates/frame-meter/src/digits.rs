use std::sync::OnceLock;

use crate::constants::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W};

const DIGIT_COUNT: usize = 10;
const PADDED_DIGIT_COUNT: usize = 12;

// Valid scores are initialized at -1.0, so this remains distinguishable even
// when every template alignment scores below zero.
pub(crate) const UNCOMPUTED_CORRELATION: f32 = -2.0;

// 実ゲームを撮影した動画サンプルから生成した、セルごとの正規化済み画素統計。
// 元動画、frame、cropは含まず、数字相関に必要なf32値だけを保持する。
static DIGIT_TEMPLATE_BYTES: &[u8] = include_bytes!("data/meter_digits.bin");

fn digit_templates() -> Option<&'static [f32]> {
    let stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    let expected = DIGIT_COUNT * stride * 4;
    if DIGIT_TEMPLATE_BYTES.len() != expected {
        return None;
    }
    static CACHE: OnceLock<Vec<f32>> = OnceLock::new();
    let templates = CACHE.get_or_init(|| {
        let digit_major = DIGIT_TEMPLATE_BYTES
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect::<Vec<_>>();
        let mut pixel_major = vec![0.0; stride * PADDED_DIGIT_COUNT];
        for pixel in 0..stride {
            for digit in 0..DIGIT_COUNT {
                pixel_major[pixel * PADDED_DIGIT_COUNT + digit] =
                    digit_major[digit * stride + pixel];
            }
        }
        pixel_major
    });
    Some(templates.as_slice())
}

pub(crate) fn digit_correlations(cells_vpatch: &[f32]) -> Option<Vec<[f32; 10]>> {
    let stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    if cells_vpatch.len() != CELL_COUNT * stride {
        return None;
    }
    let templates = digit_templates()?;
    let height = DIGIT_TEMPLATE_H;
    let width = DIGIT_TEMPLATE_W;
    let mut best = vec![[-1.0f32; 10]; CELL_COUNT];

    for dy in [-1i32, 0, 1] {
        let source_y = dy.max(0) as usize;
        let template_y = (-dy).max(0) as usize;
        let row_count = height - dy.unsigned_abs() as usize;

        for dx in [-2i32, -1, 0, 1, 2] {
            let source_x = dx.max(0) as usize;
            let template_x = (-dx).max(0) as usize;
            let column_count = width - dx.unsigned_abs() as usize;

            let area = (row_count * column_count) as f32;
            for cell_index in 0..CELL_COUNT {
                let patch = &cells_vpatch[cell_index * stride..(cell_index + 1) * stride];
                update_best_scores(
                    &mut best[cell_index],
                    patch,
                    templates,
                    source_y,
                    source_x,
                    template_y,
                    template_x,
                    row_count,
                    column_count,
                    area,
                );
            }
        }
    }
    Some(best)
}

#[cfg(test)]
pub(crate) fn digit_correlations_for_cells(
    cells_vpatch: &[f32],
    cell_indices: impl IntoIterator<Item = usize>,
) -> Option<Vec<[f32; 10]>> {
    let cells = cell_indices
        .into_iter()
        .filter(|&index| index < CELL_COUNT)
        .collect::<Vec<_>>();
    let stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    if cells_vpatch.len() != CELL_COUNT * stride {
        return None;
    }
    digit_correlations_for_cell_patches(cells_vpatch, &cells, false)
}

pub(crate) fn digit_correlations_for_compact_patches(
    cells_vpatch: &[f32],
    cell_indices: &[usize],
) -> Option<Vec<[f32; 10]>> {
    let stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    if cell_indices.iter().any(|&index| index >= CELL_COUNT)
        || cells_vpatch.len() != cell_indices.len() * stride
    {
        return None;
    }
    digit_correlations_for_cell_patches(cells_vpatch, cell_indices, true)
}

fn digit_correlations_for_cell_patches(
    cells_vpatch: &[f32],
    cells: &[usize],
    compact: bool,
) -> Option<Vec<[f32; 10]>> {
    let templates = digit_templates()?;
    let height = DIGIT_TEMPLATE_H;
    let width = DIGIT_TEMPLATE_W;
    let stride = height * width;
    let mut best = vec![[UNCOMPUTED_CORRELATION; 10]; CELL_COUNT];
    for &cell_index in cells {
        best[cell_index] = [-1.0; 10];
    }

    for dy in [-1i32, 0, 1] {
        let source_y = dy.max(0) as usize;
        let template_y = (-dy).max(0) as usize;
        let row_count = height - dy.unsigned_abs() as usize;

        for dx in [-2i32, -1, 0, 1, 2] {
            let source_x = dx.max(0) as usize;
            let template_x = (-dx).max(0) as usize;
            let column_count = width - dx.unsigned_abs() as usize;

            let area = (row_count * column_count) as f32;
            for (patch_position, &cell_index) in cells.iter().enumerate() {
                let patch_index = if compact { patch_position } else { cell_index };
                let patch = &cells_vpatch[patch_index * stride..(patch_index + 1) * stride];
                update_best_scores(
                    &mut best[cell_index],
                    patch,
                    templates,
                    source_y,
                    source_x,
                    template_y,
                    template_x,
                    row_count,
                    column_count,
                    area,
                );
            }
        }
    }
    Some(best)
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn update_best_scores(
    best: &mut [f32; DIGIT_COUNT],
    patch: &[f32],
    templates: &[f32],
    source_y: usize,
    source_x: usize,
    template_y: usize,
    template_x: usize,
    row_count: usize,
    column_count: usize,
    area: f32,
) {
    let sums = alignment_sums(
        patch,
        templates,
        source_y,
        source_x,
        template_y,
        template_x,
        row_count,
        column_count,
    );
    for (best_score, sum) in best.iter_mut().zip(sums) {
        *best_score = best_score.max(sum / area);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn alignment_sums(
    patch: &[f32],
    templates: &[f32],
    source_y: usize,
    source_x: usize,
    template_y: usize,
    template_x: usize,
    row_count: usize,
    column_count: usize,
) -> [f32; DIGIT_COUNT] {
    let mut sums = [0.0; DIGIT_COUNT];
    for row in 0..row_count {
        for column in 0..column_count {
            let source_index = (source_y + row) * DIGIT_TEMPLATE_W + source_x + column;
            let template_index = (template_y + row) * DIGIT_TEMPLATE_W + template_x + column;
            let value = patch[source_index];
            let template_start = template_index * PADDED_DIGIT_COUNT;
            for digit in 0..DIGIT_COUNT {
                sums[digit] += value * templates[template_start + digit];
            }
        }
    }
    sums
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn alignment_sums(
    patch: &[f32],
    templates: &[f32],
    source_y: usize,
    source_x: usize,
    template_y: usize,
    template_x: usize,
    row_count: usize,
    column_count: usize,
) -> [f32; DIGIT_COUNT] {
    use core::arch::wasm32::{f32x4_add, f32x4_mul, f32x4_splat, v128_load, v128_store};

    let mut sums0 = f32x4_splat(0.0);
    let mut sums1 = f32x4_splat(0.0);
    let mut sums2 = f32x4_splat(0.0);
    for row in 0..row_count {
        for column in 0..column_count {
            let source_index = (source_y + row) * DIGIT_TEMPLATE_W + source_x + column;
            let template_index = (template_y + row) * DIGIT_TEMPLATE_W + template_x + column;
            let template_start = template_index * PADDED_DIGIT_COUNT;
            // SAFETY: callers pass one complete digit patch and the padded
            // template cache. Every computed source/template index stays
            // within those fixed 26x13 layouts.
            unsafe {
                let value = f32x4_splat(*patch.get_unchecked(source_index));
                let template = templates.as_ptr().add(template_start);
                sums0 = f32x4_add(sums0, f32x4_mul(value, v128_load(template.cast())));
                sums1 = f32x4_add(sums1, f32x4_mul(value, v128_load(template.add(4).cast())));
                sums2 = f32x4_add(sums2, f32x4_mul(value, v128_load(template.add(8).cast())));
            }
        }
    }

    let mut padded = [0.0; PADDED_DIGIT_COUNT];
    // SAFETY: each store writes four f32 lanes into the corresponding
    // non-overlapping portion of the 12-element output.
    unsafe {
        v128_store(padded.as_mut_ptr().cast(), sums0);
        v128_store(padded.as_mut_ptr().add(4).cast(), sums1);
        v128_store(padded.as_mut_ptr().add(8).cast(), sums2);
    }
    [
        padded[0], padded[1], padded[2], padded[3], padded[4], padded[5], padded[6], padded[7],
        padded[8], padded[9],
    ]
}
