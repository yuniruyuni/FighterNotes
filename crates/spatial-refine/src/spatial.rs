//! Low-cost spatial observations from short candidate windows.
//!
//! This module intentionally does not identify characters or moves. It uses
//! frame-to-frame motion to provide normalized player anchors and small moving
//! object candidates. Callers should only run it around event windows and
//! combine the result with input and frame-meter evidence.

mod config;
mod error;
mod extraction;
mod model;
mod parameters;
mod refinement;
#[cfg(test)]
mod tests;
mod window_planning;

pub use config::SpatialConfig;
pub use error::SpatialError;
pub use extraction::SpatialExtractor;
pub use model::{
    ActorHint, ActorObservation, CameraMotion, ContactObservation, DistanceBand, HorizontalMotion,
    HorizontalOrder, MotionRegionObservation, ProjectileCandidate, SpatialHints,
    SpatialObservation, SpatialPoint, SpatialRect,
};
pub use refinement::refine_match_events_with_spatial;
pub use window_planning::{
    spatial_candidate_windows, SpatialCandidateWindow, SpatialFrameRange, SpatialHintRange,
};
