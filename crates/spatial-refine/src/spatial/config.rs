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
    /// Minimum effect-colored changed cells for a region to count as a
    /// contact spark on a hinted contact frame.
    pub contact_min_effect_cells: u32,
    /// Minimum fraction of effect-colored cells among changed cells. Rejects
    /// body motion that merely brushes a bright costume.
    pub contact_min_effect_fraction: f32,
    /// Horizontal slack around the tracked actor span when locating a spark.
    pub contact_actor_pad: f32,
    /// A single motion region covering at least this fraction of the
    /// playfield area is a scene disruption (super flash, cinematic cut,
    /// round transition). Nothing meaningful can be localized in it.
    /// 大技のヒット VFX は本体と合体して面積の半分程度まで届くため、
    /// それを演出と混同しない値にする。
    pub disruption_min_area: f32,
    /// After a hinted contact frame was disrupted, keep looking for the
    /// spark this many extra frames past the hint, refreshed while the
    /// disruption continues.
    pub contact_disruption_grace: u32,
    /// Minimum spark cells for an embedded spark: one that merged into a
    /// body region because real hitstop still shakes the bodies. Costume
    /// patches are smaller or more dispersed than this.
    pub contact_embedded_min_cells: u32,
    /// Maximum spatial spread (std dev, normalized) of the spark cells for
    /// an embedded spark. Sparks are compact; costume highlights follow the
    /// body and spread wider.
    pub contact_embedded_max_spread: f32,
    /// On a hinted contact frame, a region at least this effect-colored is
    /// treated as VFX and never assigned to an actor track. Hitstop freezes
    /// both bodies, so their regions cannot appear on such frames.
    pub contact_effect_max_actor_fraction: f32,
    /// Top of the floor band (normalized y) searched for ground shadows.
    /// Kept below the frame-meter overlay so the band stays visible.
    pub shadow_band_top: f32,
    /// Bottom of the floor band searched for ground shadows.
    pub shadow_band_bottom: f32,
    /// How much darker than the row median a cell must be to count as
    /// shadow. Relative to the local floor, so stage brightness cancels out.
    pub shadow_min_contrast: u8,
    /// Minimum shadowed columns for a cluster to become a candidate.
    pub shadow_min_cells: u32,
    /// Maximum horizontal distance between a track anchor and a shadow
    /// centroid for the anchor to snap onto the shadow.
    pub shadow_snap_dx: f32,
    /// A region at least this wide that spans both track anchors is treated
    /// as the two bodies merged into one motion blob. Its center is between
    /// the players, so each track keeps its own x instead of moving there.
    pub merged_region_min_width: f32,
    /// Confidence reported when a track is updated from a merged region.
    pub merged_region_confidence: f32,
    /// How decisively the swapped signature pairing must beat the straight
    /// pairing before the left-is-P1 assumption is overridden at window
    /// initialization. 2.0 requires the swapped total color distance to be
    /// at most half of the straight one.
    pub signature_swap_margin: f32,
    /// Fraction of appearance cells that must stay under `motion_threshold`
    /// for an unobserved track to be confirmed as a stationary actor instead
    /// of decaying. Guarding, downed and frozen actors produce no motion, so
    /// an unchanged appearance is positive evidence, not absence of it.
    pub still_match_min_fraction: f32,
    /// Confidence reported for a stillness-confirmed frame. Below a fresh
    /// motion observation, far above a blind carry-forward.
    pub still_confidence: f32,
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
            actor_ground_y: 0.70,
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
            contact_min_effect_cells: 3,
            contact_min_effect_fraction: 0.30,
            contact_actor_pad: 0.10,
            disruption_min_area: 0.75,
            contact_disruption_grace: 12,
            contact_embedded_min_cells: 6,
            contact_embedded_max_spread: 0.05,
            contact_effect_max_actor_fraction: 0.60,
            shadow_band_top: 0.87,
            shadow_band_bottom: 0.96,
            shadow_min_contrast: 12,
            shadow_min_cells: 2,
            shadow_snap_dx: 0.06,
            merged_region_min_width: 0.24,
            merged_region_confidence: 0.62,
            signature_swap_margin: 2.0,
            still_match_min_fraction: 0.90,
            still_confidence: 0.68,
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
