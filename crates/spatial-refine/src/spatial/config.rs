use serde::{Deserialize, Serialize};

use super::SpatialRect;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpatialConfig {
    /// Size of one motion-analysis cell in source pixels.
    pub cell_size: u32,
    /// Minimum maximum-channel difference for a cell to count as motion.
    pub motion_threshold: u8,
    /// Minimum number of active neighbors used to reject isolated noise.
    pub min_motion_neighbors: u8,
    pub playfield: SpatialRect,
    pub excluded_regions: Vec<SpatialRect>,
    pub actor_min_changed_cells: u32,
    pub actor_min_height: f32,
    pub actor_ground_y: f32,
    pub max_track_dx: f32,
    pub max_track_dy: f32,
    pub region_merge_gap: f32,
    pub projectile_min_changed_cells: u32,
    pub projectile_max_changed_cells: u32,
    pub projectile_max_width: f32,
    pub projectile_max_height: f32,
    pub projectile_min_y: f32,
    pub projectile_max_y: f32,
    pub actor_exclusion_dx: f32,
    pub projectile_match_distance: f32,
    pub projectile_max_track_gap: u32,
    pub max_projectile_candidates: usize,
    pub overlap_distance: f32,
    pub close_distance: f32,
    pub mid_distance: f32,
    pub max_stale_frames: u32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            cell_size: 8,
            motion_threshold: 18,
            min_motion_neighbors: 2,
            playfield: SpatialRect::new(0.0, 0.14, 1.0, 0.97),
            excluded_regions: Vec::new(),
            actor_min_changed_cells: 10,
            actor_min_height: 0.065,
            actor_ground_y: 0.84,
            max_track_dx: 0.18,
            max_track_dy: 0.30,
            region_merge_gap: 0.045,
            projectile_min_changed_cells: 2,
            projectile_max_changed_cells: 480,
            projectile_max_width: 0.28,
            projectile_max_height: 0.24,
            projectile_min_y: 0.25,
            projectile_max_y: 0.82,
            actor_exclusion_dx: 0.11,
            projectile_match_distance: 0.16,
            projectile_max_track_gap: 8,
            max_projectile_candidates: 8,
            overlap_distance: 0.09,
            close_distance: 0.22,
            mid_distance: 0.46,
            max_stale_frames: 20,
        }
    }
}

impl SpatialConfig {
    /// Configuration for SF6 training/replay footage with input history and
    /// frame-meter overlays visible, as in the verified full-match recordings.
    pub fn sf6_training_overlay() -> Self {
        Self {
            excluded_regions: vec![
                // Input history columns. Keep the floor strip so edge-positioned
                // character feet can still contribute motion.
                SpatialRect::new(0.0, 0.14, 0.135, 0.70),
                SpatialRect::new(0.865, 0.14, 1.0, 0.70),
                // Frame-meter bars and their changing cells.
                SpatialRect::new(0.18, 0.70, 0.82, 0.86),
            ],
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_playfield_has_all_four_calibrated_edges() {
        assert_eq!(
            SpatialConfig::default().playfield,
            SpatialRect::new(0.0, 0.14, 1.0, 0.97)
        );
    }
}
