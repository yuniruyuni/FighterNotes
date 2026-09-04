mod components;
mod mask;

use super::super::{SpatialConfig, SpatialPoint, SpatialRect};
use super::grid::CellGrid;

use super::grid::CellColor;

/// 打撃ヒットの VFX。橙〜黄の暖色で、明るく彩度が高い。
/// 閾値は実録画のスパーク領域と衣装・背景のセル統計から選んだ。
pub(super) fn is_warm_effect(cell: CellColor) -> bool {
    let high = cell.r.max(cell.g).max(cell.b);
    let low = cell.r.min(cell.g).min(cell.b);
    high >= 145 && high.saturating_sub(low) >= 65
}

/// ガード時の VFX。白核+淡青で、非常に明るく低彩度かつ青が赤より強い。
/// トレモステージの明るい壁は warm-gray (b < r) なので、この条件は
/// 衣装や背景の明部と分離できる(実測: 壁 hi≤240 b-r≈-18、スパーク
/// hi≥250 b-r≈+44)。
pub(super) fn is_cold_effect(cell: CellColor) -> bool {
    let high = cell.r.max(cell.g).max(cell.b);
    let low = cell.r.min(cell.g).min(cell.b);
    high >= 235 && high.saturating_sub(low) < 65 && cell.b > cell.r
}

#[derive(Clone, Debug)]
pub(super) struct MotionRegion {
    pub(super) bounds: SpatialRect,
    pub(super) changed_cells: u32,
    pub(super) energy: u64,
    pub(super) effect_cells: u32,
    /// Normalized coordinate sums of effect-colored cell centers. Divided by
    /// `effect_cells` they give the spark centroid, which is a better contact
    /// point than the bounding-box center once body motion joins the region.
    pub(super) effect_x_sum: f32,
    pub(super) effect_y_sum: f32,
    pub(super) effect_xx_sum: f32,
    pub(super) effect_yy_sum: f32,
    /// Cold (guard-spark) effect cells and their coordinate sums, kept apart
    /// from the warm counters so projectile confidence keeps its meaning.
    pub(super) cold_effect_cells: u32,
    pub(super) cold_x_sum: f32,
    pub(super) cold_y_sum: f32,
    pub(super) cold_xx_sum: f32,
    pub(super) cold_yy_sum: f32,
    /// Sums of the current cell colors over changed cells, for the mean
    /// region color used by player identity signatures.
    pub(super) color_sum: [u64; 3],
}

impl MotionRegion {
    pub(super) fn center(&self) -> SpatialPoint {
        self.bounds.center()
    }

    /// 変化セル上の現在色の平均。プレイヤー同定のシグネチャに使う。
    pub(super) fn mean_color(&self) -> [f32; 3] {
        let cells = self.changed_cells.max(1) as f32;
        [
            self.color_sum[0] as f32 / cells,
            self.color_sum[1] as f32 / cells,
            self.color_sum[2] as f32 / cells,
        ]
    }

    /// Warm + cold を合わせたスパークセル数。contact の証拠に使う。
    pub(super) fn spark_cells(&self) -> u32 {
        self.effect_cells + self.cold_effect_cells
    }

    /// Warm + cold を合わせたスパーク重心。
    pub(super) fn spark_centroid(&self) -> Option<SpatialPoint> {
        let cells = self.spark_cells();
        if cells == 0 {
            return None;
        }
        Some(SpatialPoint::new(
            (self.effect_x_sum + self.cold_x_sum) / cells as f32,
            (self.effect_y_sum + self.cold_y_sum) / cells as f32,
        ))
    }

    /// スパークセルの空間的な広がり(x/y 標準偏差の二乗和の平方根)。
    /// スパークは凝集し、衣装の明色は体に沿って分散する。
    pub(super) fn spark_spread(&self) -> Option<f32> {
        let cells = self.spark_cells();
        let centroid = self.spark_centroid()?;
        let mean_xx = (self.effect_xx_sum + self.cold_xx_sum) / cells as f32;
        let mean_yy = (self.effect_yy_sum + self.cold_yy_sum) / cells as f32;
        let variance_x = (mean_xx - centroid.x * centroid.x).max(0.0);
        let variance_y = (mean_yy - centroid.y * centroid.y).max(0.0);
        Some((variance_x + variance_y).sqrt())
    }

    pub(super) fn anchor(&self) -> SpatialPoint {
        SpatialPoint::new(
            (self.bounds.left + self.bounds.right) * 0.5,
            self.bounds.bottom,
        )
    }

    fn merge(&mut self, other: &Self) {
        self.bounds = self.bounds.union(other.bounds);
        self.changed_cells += other.changed_cells;
        self.energy += other.energy;
        self.effect_cells += other.effect_cells;
        self.effect_x_sum += other.effect_x_sum;
        self.effect_y_sum += other.effect_y_sum;
        self.effect_xx_sum += other.effect_xx_sum;
        self.effect_yy_sum += other.effect_yy_sum;
        self.cold_effect_cells += other.cold_effect_cells;
        self.cold_x_sum += other.cold_x_sum;
        self.cold_y_sum += other.cold_y_sum;
        self.cold_xx_sum += other.cold_xx_sum;
        self.cold_yy_sum += other.cold_yy_sum;
        self.color_sum[0] += other.color_sum[0];
        self.color_sum[1] += other.color_sum[1];
        self.color_sum[2] += other.color_sum[2];
    }
}

pub(super) fn regions(
    previous: &CellGrid,
    current: &CellGrid,
    source_width: u32,
    source_height: u32,
    config: &SpatialConfig,
) -> Vec<MotionRegion> {
    let mask = mask::motion_mask(previous, current, config);
    let regions = components::connected_regions(
        &mask,
        current,
        source_width,
        source_height,
        config.cell_size,
    );
    components::merge_nearby(regions, config.region_merge_gap)
}

pub(super) fn actor_candidate(region: &MotionRegion, config: &SpatialConfig) -> bool {
    region.changed_cells >= config.actor_min_changed_cells
        && region.bounds.height() >= config.actor_min_height
        && region.bounds.width() <= 0.42
        && region.bounds.height() <= 0.78
}

pub(super) fn projectile_candidate(region: &MotionRegion, config: &SpatialConfig) -> bool {
    let center_y = region.center().y;
    region.changed_cells >= config.projectile_min_changed_cells
        && region.changed_cells <= config.projectile_max_changed_cells
        && region.bounds.width() <= config.projectile_max_width
        && region.bounds.height() <= config.projectile_max_height
        && center_y >= config.projectile_min_y
        && center_y <= config.projectile_max_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(r: u8, g: u8, b: u8) -> CellColor {
        CellColor { r, g, b }
    }

    /// 暖色 VFX は「明度 145 以上かつ彩度差 65 以上」。両方の境界を含む。
    #[test]
    fn warm_effect_boundaries_are_inclusive() {
        assert!(is_warm_effect(cell(145, 80, 80)));
        assert!(!is_warm_effect(cell(144, 79, 79)));
        assert!(!is_warm_effect(cell(145, 81, 81)));
        // 明度はどのチャネルが最大でも同じに扱う。
        assert!(is_warm_effect(cell(80, 145, 80)));
        assert!(is_warm_effect(cell(80, 80, 145)));
    }

    #[test]
    fn mean_color_averages_each_channel_over_changed_cells() {
        let region = MotionRegion {
            bounds: SpatialRect::new(0.0, 0.0, 0.1, 0.1),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [250, 320, 490],
        };
        assert_eq!(region.mean_color(), [2.5, 3.2, 4.9]);
    }

    /// 寒色 VFX は「明度 235 以上・彩度差 65 未満・青が赤より強い」。
    /// warm-gray の壁 (b < r) や彩度の高い衣装を通さない。
    #[test]
    fn cold_effect_needs_bright_low_saturation_blue() {
        assert!(is_cold_effect(cell(200, 220, 235)));
        assert!(!is_cold_effect(cell(200, 219, 234)), "明度 234 は暗すぎる");
        assert!(
            !is_cold_effect(cell(170, 200, 235)),
            "彩度差 65 は寒色ではない"
        );
        assert!(is_cold_effect(cell(171, 200, 235)));
        assert!(!is_cold_effect(cell(235, 235, 235)), "b == r は青くない");
        assert!(is_cold_effect(cell(234, 235, 235)));
        assert!(!is_cold_effect(cell(255, 250, 240)), "warm-gray の明部");
        // 明度は緑が最大でも同じに扱う(緑白のフラッシュ)。
        assert!(is_cold_effect(cell(180, 240, 200)));
    }

    /// 判定は排他で、彩度差 65 を境に暖色と寒色へ分かれる。
    #[test]
    fn warm_and_cold_are_disjoint() {
        let warm = cell(255, 190, 40);
        let cold = cell(245, 250, 255);
        assert!(is_warm_effect(warm) && !is_cold_effect(warm));
        assert!(is_cold_effect(cold) && !is_warm_effect(cold));
    }
}
