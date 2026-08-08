use serde::{Deserialize, Serialize};

use super::{DistanceBand, HorizontalMotion, HorizontalOrder, SpatialPoint, SpatialRect};

/// A player location carried through a short candidate window.
///
/// `observed == false` means the last anchor was carried forward because no
/// compatible motion region was found in this frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActorObservation {
    pub anchor: SpatialPoint,
    pub bounds: SpatialRect,
    pub confidence: f32,
    pub observed: bool,
    /// The selected region reaches the configured ground band. False can mean
    /// either an airborne actor or a carried upper-body observation.
    #[serde(default)]
    pub ground_anchor: bool,
    pub discontinuity: bool,
}

/// A small moving region which may be a projectile.
///
/// This is deliberately named a candidate: stage animation, hit effects and
/// training overlays can also produce compact motion. A trajectory confirmed
/// over multiple frames is stronger evidence than a single observation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectileCandidate {
    pub track_id: u32,
    pub center: SpatialPoint,
    pub bounds: SpatialRect,
    pub velocity_x: Option<f32>,
    pub motion: HorizontalMotion,
    pub trajectory_confirmed: bool,
    pub confidence: f32,
}

/// Compact diagnostics for every connected motion region in a candidate frame.
/// This allows rejected-candidate inspection without retaining source pixels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MotionRegionObservation {
    pub bounds: SpatialRect,
    pub changed_cells: u32,
    pub mean_delta: f32,
    pub effect_color_fraction: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialObservation {
    pub frame_index: u32,
    pub p1: Option<ActorObservation>,
    pub p2: Option<ActorObservation>,
    pub screen_distance: Option<f32>,
    pub distance_band: Option<DistanceBand>,
    pub horizontal_order: Option<HorizontalOrder>,
    pub projectile_candidates: Vec<ProjectileCandidate>,
    #[serde(default)]
    pub motion_regions: Vec<MotionRegionObservation>,
}
