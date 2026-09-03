mod components;
mod mask;

use super::super::{SpatialConfig, SpatialPoint, SpatialRect};
use super::grid::CellGrid;

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
}

impl MotionRegion {
    pub(super) fn center(&self) -> SpatialPoint {
        self.bounds.center()
    }

    pub(super) fn effect_centroid(&self) -> Option<SpatialPoint> {
        if self.effect_cells == 0 {
            return None;
        }
        Some(SpatialPoint::new(
            self.effect_x_sum / self.effect_cells as f32,
            self.effect_y_sum / self.effect_cells as f32,
        ))
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
