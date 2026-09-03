mod actors;
mod camera;
mod contact;
mod grid;
mod motion;
mod projectiles;
mod relationship;
mod shadows;

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
    previous_strips: Option<camera::CameraStrips>,
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
            previous_strips: None,
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
        self.previous_strips = None;
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
        let shadow_candidates = shadows::shadow_candidates(&grid, &self.config);
        let tracked = self.actors.observe(
            frame_index,
            &regions,
            &grid,
            &shadow_candidates,
            hints,
            &self.config,
        );
        let projectile_candidates = self.projectiles.observe(
            frame_index,
            &regions,
            &tracked.used_regions,
            [tracked.p1.as_ref(), tracked.p2.as_ref()],
            &self.config,
        );
        let (screen_distance, distance_band, horizontal_order) =
            spatial_relationship(tracked.p1.as_ref(), tracked.p2.as_ref(), &self.config);
        let strips = camera::sample_strips(rgba, width, height);
        let contact = contact::contact_observation(
            &regions,
            &tracked.used_regions,
            tracked.p1.as_ref(),
            tracked.p2.as_ref(),
            hints.contact_effect,
            &self.config,
        );
        let masked_centers = camera::masked_centers(tracked.p1.as_ref(), tracked.p2.as_ref());
        let camera_motion = self
            .previous_strips
            .as_ref()
            .and_then(|previous| camera::estimate(previous, &strips, &masked_centers));
        self.previous_strips = Some(strips);
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
            contact,
            camera: camera_motion,
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

    /// 320x180。静的なテクスチャ背景の上を、テクスチャ付きの 2 体が
    /// 水平移動する。背景は動かないので、正しいカメラ推定は常に静止。
    fn textured_scene(actor1_x: i32, actor2_x: i32) -> Vec<u8> {
        const SCENE_WIDTH: usize = 320;
        const SCENE_HEIGHT: usize = 180;
        let mut rgba = vec![255u8; SCENE_WIDTH * SCENE_HEIGHT * 4];
        let inside = |left: i32, x: usize| {
            let dx = x as i32 - left;
            (0..80).contains(&dx)
        };
        for y in 0..SCENE_HEIGHT {
            for x in 0..SCENE_WIDTH {
                // 影閾値(12)未満の振幅で、相関には十分なテクスチャ。
                let mut value = 40 + ((x * 7 + y * 13) % 11) as u8;
                if (28..108).contains(&y) {
                    // 4px ブロックのテクスチャ。4px の移動でブロック 1 個ぶん
                    // ずれ、全セルが確実に motion 閾値を越える。
                    if inside(actor1_x, x) {
                        value = 120 + (((x as i32 - actor1_x) / 4 * 23) % 90) as u8;
                    } else if inside(actor2_x, x) {
                        value = 120 + (((x as i32 - actor2_x) / 4 * 29) % 87) as u8;
                    }
                }
                let index = (y * SCENE_WIDTH + x) * 4;
                rgba[index] = value;
                rgba[index + 1] = value;
                rgba[index + 2] = value;
            }
        }
        rgba
    }

    /// カメラは背景だけを見る。本体(と strip 行にかかるその動き)は
    /// マスクされるので、静止した背景に対する推定は静止のまま。
    #[test]
    fn camera_estimate_ignores_actor_motion() {
        let mut extractor = SpatialExtractor::new(SpatialConfig::default());
        extractor
            .observe_rgba(
                1,
                &textured_scene(80, 224),
                320,
                180,
                SpatialHints::default(),
            )
            .unwrap();
        let observed = extractor
            .observe_rgba(
                2,
                &textured_scene(84, 228),
                320,
                180,
                SpatialHints::default(),
            )
            .unwrap();
        // マスクの位置が想定どおりであることも含めて検査する。
        let p1 = observed.p1.expect("P1 track");
        let p2 = observed.p2.expect("P2 track");
        assert!((p1.anchor.x - 0.3875).abs() < 0.03, "{p1:?}");
        assert!((p2.anchor.x - 0.8313).abs() < 0.03, "{p2:?}");
        let camera = observed.camera.expect("camera motion");
        assert!((camera.pan_dx * 320.0).abs() < 0.5, "{camera:?}");
        assert!((camera.zoom_ratio - 1.0).abs() < 0.01, "{camera:?}");
    }

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
