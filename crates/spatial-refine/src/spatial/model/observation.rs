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

/// A hit-effect (spark) location observed during a hinted contact frame.
///
/// During hitstop both bodies and the camera freeze, so frame differencing
/// isolates the effect. The centroid of bright, saturated changed cells is
/// the best single-point estimate of where the attack connected. This is
/// evidence about the contact already confirmed by the first stage, never a
/// contact detection of its own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactObservation {
    /// Centroid of the effect-colored changed cells, normalized 0..1.
    pub center: SpatialPoint,
    pub bounds: SpatialRect,
    pub effect_cells: u32,
    pub confidence: f32,
}

/// Camera motion between the previous and the current frame, estimated by
/// correlating world-anchored background strips. `pan_dx` is the horizontal
/// shift of the background in normalized screen x (positive = the camera
/// moved left), `zoom_ratio` the frame-to-frame scale factor. Integrating
/// these over a window relates screen distance to game distance and reveals
/// the pan clamp near a stage wall.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraMotion {
    pub pan_dx: f32,
    pub zoom_ratio: f32,
    pub confidence: f32,
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
    /// Present only when the frame carried a contact hint and an effect
    /// region passed the spark criteria.
    #[serde(default)]
    pub contact: Option<ContactObservation>,
    /// Camera motion versus the previous frame, when it could be estimated.
    #[serde(default)]
    pub camera: Option<CameraMotion>,
}
