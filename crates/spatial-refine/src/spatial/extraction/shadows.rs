//! Ground-shadow detection in the floor band.
//!
//! SF6 renders a contact shadow under each character. Unlike frame-to-frame
//! motion the shadow is present on every frame, including for a guarding or
//! downed actor, and it stays visible inside the input-history overlay
//! columns because only the floor strip below the frame-meter band is
//! searched. A shadow cluster therefore provides a per-frame horizontal
//! anchor that survives stillness, close-range region merging and overlay
//! exclusions.
//!
//! Detection is relative to the local floor: a cell counts as shadow when it
//! is darker than its row median by a configured contrast, so stage
//! brightness and floor texture largely cancel out.

use super::grid::CellGrid;
use super::SpatialConfig;

#[derive(Clone, Copy, Debug)]
pub(super) struct ShadowCandidate {
    pub(super) center_x: f32,
    pub(super) weight: f32,
}

pub(super) fn shadow_candidates(grid: &CellGrid, config: &SpatialConfig) -> Vec<ShadowCandidate> {
    if grid.width == 0 || grid.height == 0 {
        return Vec::new();
    }
    let row_top = (config.shadow_band_top * grid.height as f32)
        .floor()
        .max(0.0) as usize;
    let row_bottom =
        ((config.shadow_band_bottom * grid.height as f32).ceil() as usize).min(grid.height);
    if row_bottom <= row_top {
        return Vec::new();
    }

    // 列ごとに「行 median からの暗さ」を積む。
    let mut darkness = vec![0u32; grid.width];
    for y in row_top..row_bottom {
        let mut lumas: Vec<u8> = (0..grid.width).map(|x| luma(grid, x, y)).collect();
        let row = lumas.clone();
        let median_index = lumas.len() / 2;
        let (_, median, _) = lumas.select_nth_unstable(median_index);
        let median = *median;
        for (x, value) in row.iter().enumerate() {
            let contrast = median.saturating_sub(*value);
            if contrast >= config.shadow_min_contrast {
                darkness[x] += contrast as u32;
            }
        }
    }

    // 1 セルの隙間まで許して連続した暗い列をクラスタにする。
    let mut candidates = Vec::new();
    let mut run_start: Option<usize> = None;
    let mut gap = 0usize;
    let flush = |start: usize, end: usize, candidates: &mut Vec<ShadowCandidate>| {
        let cells = darkness[start..end].iter().filter(|&&d| d > 0).count() as u32;
        if cells < config.shadow_min_cells {
            return;
        }
        let weight: u32 = darkness[start..end].iter().sum();
        let weighted_x: f32 = darkness[start..end]
            .iter()
            .enumerate()
            .map(|(offset, &d)| (start + offset) as f32 * d as f32)
            .sum();
        candidates.push(ShadowCandidate {
            center_x: (weighted_x / weight as f32 + 0.5) / grid.width as f32,
            weight: weight as f32,
        });
    };
    for (x, &column_darkness) in darkness.iter().enumerate() {
        if column_darkness > 0 {
            if run_start.is_none() {
                run_start = Some(x);
            }
            gap = 0;
        } else if let Some(start) = run_start {
            gap += 1;
            if gap > 1 {
                flush(start, x + 1 - gap, &mut candidates);
                run_start = None;
                gap = 0;
            }
        }
    }
    if let Some(start) = run_start {
        flush(start, grid.width, &mut candidates);
    }
    candidates.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    candidates.truncate(4);
    candidates
}

fn luma(grid: &CellGrid, x: usize, y: usize) -> u8 {
    let cell = grid.cells[y * grid.width + x];
    ((cell.r as u16 + cell.g as u16 + cell.b as u16) / 3) as u8
}
