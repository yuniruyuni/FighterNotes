mod actors;
mod grid;
mod motion;
mod projectiles;
mod relationship;

use actors::ActorTracker;
use grid::{validate_rgba, CellGrid};
use projectiles::ProjectileTracker;
use relationship::spatial_relationship;

use super::{
    MotionRegionObservation, SpatialConfig, SpatialError, SpatialHints, SpatialObservation,
};

/// Stateful extractor intended for one short event window.
pub struct SpatialExtractor {
    config: SpatialConfig,
    dimensions: Option<(u32, u32)>,
    previous_grid: Option<CellGrid>,
    actors: ActorTracker,
    projectiles: ProjectileTracker,
}

impl SpatialExtractor {
    pub fn new(mut config: SpatialConfig) -> Self {
        config.cell_size = config.cell_size.max(1);
        Self {
            config,
            dimensions: None,
            previous_grid: None,
            actors: ActorTracker::default(),
            projectiles: ProjectileTracker::default(),
        }
    }

    pub fn config(&self) -> &SpatialConfig {
        &self.config
    }

    pub fn reset(&mut self) {
        self.dimensions = None;
        self.previous_grid = None;
        self.actors.reset();
        self.projectiles.reset();
    }

    pub fn observe_rgba(
        &mut self,
        frame_index: u32,
        rgba: &[u8],
        width: u32,
        height: u32,
        hints: SpatialHints,
    ) -> Result<SpatialObservation, SpatialError> {
        validate_rgba(rgba, width, height)?;
        self.validate_dimensions(width, height)?;

        let grid = CellGrid::from_rgba(rgba, width, height, self.config.cell_size);
        let regions = self
            .previous_grid
            .as_ref()
            .map(|previous| motion::regions(previous, &grid, width, height, &self.config))
            .unwrap_or_default();
        let tracked = self
            .actors
            .observe(frame_index, &regions, hints, &self.config);
        let projectile_candidates = self.projectiles.observe(
            frame_index,
            &regions,
            &tracked.used_regions,
            [tracked.p1.as_ref(), tracked.p2.as_ref()],
            &self.config,
        );
        let (screen_distance, distance_band, horizontal_order) =
            spatial_relationship(tracked.p1.as_ref(), tracked.p2.as_ref(), &self.config);
        let motion_regions = regions
            .iter()
            .map(|region| MotionRegionObservation {
                bounds: region.bounds,
                changed_cells: region.changed_cells,
                mean_delta: region.energy as f32 / region.changed_cells.max(1) as f32,
                effect_color_fraction: region.effect_cells as f32
                    / region.changed_cells.max(1) as f32,
            })
            .collect();

        self.previous_grid = Some(grid);
        Ok(SpatialObservation {
            frame_index,
            p1: tracked.p1,
            p2: tracked.p2,
            screen_distance,
            distance_band,
            horizontal_order,
            projectile_candidates,
            motion_regions,
        })
    }

    fn validate_dimensions(&mut self, width: u32, height: u32) -> Result<(), SpatialError> {
        if let Some((expected_width, expected_height)) = self.dimensions {
            if (width, height) != (expected_width, expected_height) {
                return Err(SpatialError::DimensionsChanged {
                    expected_width,
                    expected_height,
                    actual_width: width,
                    actual_height: height,
                });
            }
        } else {
            self.dimensions = Some((width, height));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{blank_frame, hints, rect, test_config, HEIGHT, WIDTH};

    #[test]
    fn dimension_validation_stores_both_axes_and_reports_each_change() {
        let mut extractor = SpatialExtractor::new(SpatialConfig::default());
        assert!(extractor.validate_dimensions(320, 180).is_ok());
        assert_eq!(extractor.dimensions, Some((320, 180)));
        assert!(matches!(
            extractor.validate_dimensions(321, 180),
            Err(SpatialError::DimensionsChanged {
                expected_width: 320,
                expected_height: 180,
                actual_width: 321,
                actual_height: 180,
            })
        ));
        assert!(matches!(
            extractor.validate_dimensions(320, 181),
            Err(SpatialError::DimensionsChanged {
                expected_width: 320,
                expected_height: 180,
                actual_width: 320,
                actual_height: 181,
            })
        ));
    }

    #[test]
    fn public_extraction_excludes_small_motion_near_either_tracked_actor() {
        for x in [76, 244] {
            let mut extractor = SpatialExtractor::new(test_config());
            let first = blank_frame();
            extractor
                .observe_rgba(10, &first, WIDTH, HEIGHT, hints())
                .unwrap();

            let mut second = first.clone();
            rect(&mut second, x, 88, 8, 8, [240, 90, 20]);
            let observed = extractor
                .observe_rgba(11, &second, WIDTH, HEIGHT, SpatialHints::default())
                .unwrap();

            assert!(
                observed.projectile_candidates.is_empty(),
                "motion beside the actor at x={x} was treated as a projectile: {:?}",
                observed.projectile_candidates
            );
        }
    }
}
