use serde::{Deserialize, Serialize};

/// Per-side interval in which semantic evidence permits otherwise implausible
/// actor motion, such as a teleport discontinuity or an airborne track.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialHintRange {
    pub side: u8,
    pub start_frame: u32,
    pub end_frame: u32,
}

/// Frame interval in which the first pass confirmed a hit or block contact
/// (hitstop). Both bodies freeze there, so the extractor may read a bright,
/// saturated motion region as the contact spark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialFrameRange {
    pub start_frame: u32,
    pub end_frame: u32,
}

/// Short range selected by the deterministic first pass for spatial decoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialCandidateWindow {
    pub start_frame: u32,
    pub end_frame: u32,
    pub teleport_hints: Vec<SpatialHintRange>,
    pub airborne_hints: Vec<SpatialHintRange>,
    #[serde(default)]
    pub contact_hints: Vec<SpatialFrameRange>,
}
