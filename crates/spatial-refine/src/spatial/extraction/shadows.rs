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
    // 幅 0 は median を取れない。行の範囲は空になれば自然に何も出さない。
    if grid.width == 0 {
        return Vec::new();
    }
    let row_top = (config.shadow_band_top * grid.height as f32).floor() as usize;
    let row_bottom =
        ((config.shadow_band_bottom * grid.height as f32).ceil() as usize).min(grid.height);

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

#[cfg(test)]
mod tests {
    use super::super::grid::CellColor;
    use super::*;

    /// 20x10 グリッド。band は 0.8..1.0 → 行 8..10 の 2 行。
    fn config() -> SpatialConfig {
        SpatialConfig {
            shadow_band_top: 0.8,
            shadow_band_bottom: 1.0,
            shadow_min_contrast: 12,
            shadow_min_cells: 2,
            ..SpatialConfig::default()
        }
    }

    fn grid(width: usize, height: usize, luma: u8) -> CellGrid {
        CellGrid {
            cells: vec![
                CellColor {
                    r: luma,
                    g: luma,
                    b: luma
                };
                width * height
            ],
            width,
            height,
        }
    }

    fn set(grid: &mut CellGrid, x: usize, y: usize, color: [u8; 3]) {
        grid.cells[y * grid.width + x] = CellColor {
            r: color[0],
            g: color[1],
            b: color[2],
        };
    }

    fn darken_band(grid: &mut CellGrid, x: usize, luma: u8) {
        for y in [8, 9] {
            set(grid, x, y, [luma; 3]);
        }
    }

    #[test]
    fn only_the_floor_band_rows_are_searched() {
        // band の 1 行上に置いた暗部は影ではない。
        let mut above = grid(20, 10, 100);
        for x in [4, 5] {
            set(&mut above, x, 7, [60; 3]);
        }
        assert!(shadow_candidates(&above, &config()).is_empty());

        // band 内の同じ暗部は影になる。
        let mut inside = grid(20, 10, 100);
        for x in [4, 5] {
            darken_band(&mut inside, x, 60);
        }
        assert_eq!(shadow_candidates(&inside, &config()).len(), 1);
    }

    /// 行 median からの相対暗さで判定し、閾値ちょうどを含める。luma は
    /// RGB の平均なので、チャネルが偏った色でも同じ境界に載る。
    #[test]
    fn contrast_is_relative_to_the_row_median_and_inclusive() {
        // 平均 89 (contrast 11): 検出しない。
        let mut low = grid(20, 10, 100);
        for x in [4, 5] {
            for y in [8, 9] {
                set(&mut low, x, y, [100, 100, 67]);
            }
        }
        assert!(shadow_candidates(&low, &config()).is_empty());

        // 平均 88 (contrast 12): 検出する。
        let mut edge = grid(20, 10, 100);
        for x in [4, 5] {
            for y in [8, 9] {
                set(&mut edge, x, y, [100, 100, 64]);
            }
        }
        assert_eq!(shadow_candidates(&edge, &config()).len(), 1);
    }

    #[test]
    fn min_cells_rejects_a_single_dark_column() {
        let mut lone = grid(20, 10, 100);
        darken_band(&mut lone, 4, 60);
        assert!(shadow_candidates(&lone, &config()).is_empty());
    }

    #[test]
    fn one_column_gap_merges_and_two_column_gap_splits() {
        let mut merged = grid(20, 10, 100);
        for x in [4, 5, 7, 8] {
            darken_band(&mut merged, x, 60);
        }
        assert_eq!(shadow_candidates(&merged, &config()).len(), 1);

        let mut split = grid(20, 10, 100);
        for x in [4, 5, 8, 9] {
            darken_band(&mut split, x, 60);
        }
        let candidates = shadow_candidates(&split, &config());
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn centroid_is_darkness_weighted_and_normalized() {
        // 列 4 は contrast 12、列 5 は contrast 36。2 行で重み 24 と 72。
        let mut uneven = grid(20, 10, 100);
        darken_band(&mut uneven, 4, 88);
        darken_band(&mut uneven, 5, 64);
        let candidates = shadow_candidates(&uneven, &config());
        assert_eq!(candidates.len(), 1);
        // 重心 = (4*24 + 5*72) / 96 = 4.75。セル中心 +0.5 を幅 20 で正規化。
        assert!(
            (candidates[0].center_x - 0.2625).abs() < 1e-4,
            "{candidates:?}"
        );
        assert!((candidates[0].weight - 96.0).abs() < 1e-3, "{candidates:?}");
    }

    #[test]
    fn heaviest_four_clusters_survive_in_weight_order() {
        let mut crowded = grid(32, 10, 100);
        // 2 列空きで独立した 5 クラスタ。左から順に重みが増える。
        for (index, base) in [(0, 86), (4, 82), (8, 78), (12, 74), (16, 70)] {
            darken_band(&mut crowded, index, base);
            darken_band(&mut crowded, index + 1, base);
        }
        let candidates = shadow_candidates(&crowded, &config());
        assert_eq!(candidates.len(), 4);
        let weights: Vec<f32> = candidates.iter().map(|c| c.weight).collect();
        assert_eq!(weights, vec![120.0, 104.0, 88.0, 72.0]);
    }

    /// band の下端はグリッド高さで切り詰める前に、設定どおりの行で
    /// 止まる。0.9 なら行 9 は含まれない。
    #[test]
    fn band_bottom_stops_at_the_configured_row() {
        let narrow_band = SpatialConfig {
            shadow_band_bottom: 0.9,
            ..config()
        };
        let mut bottom_only = grid(20, 10, 100);
        for x in [4, 5] {
            set(&mut bottom_only, x, 9, [40; 3]);
        }
        assert!(shadow_candidates(&bottom_only, &narrow_band).is_empty());

        let mut in_band = grid(20, 10, 100);
        for x in [4, 5] {
            set(&mut in_band, x, 8, [40; 3]);
        }
        assert_eq!(shadow_candidates(&in_band, &narrow_band).len(), 1);
    }

    #[test]
    fn zero_width_and_uniform_grids_yield_nothing() {
        let empty = CellGrid {
            cells: Vec::new(),
            width: 0,
            height: 0,
        };
        assert!(shadow_candidates(&empty, &config()).is_empty());
        assert!(shadow_candidates(&grid(20, 10, 100), &config()).is_empty());
    }
}
