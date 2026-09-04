use serde::{Deserialize, Serialize};

use super::SpatialPoint;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActorHint {
    /// Initial or externally corrected foot/ground anchor.
    pub anchor: Option<SpatialPoint>,
    /// Permit a far-away region to replace the current track. Set this only
    /// when meter/input evidence already indicates a teleport-like action.
    pub allow_discontinuity: bool,
    /// Permit a ground track to attach to an airborne motion region. Set this
    /// from an already detected jump window; effects can otherwise pull a
    /// ground anchor upward.
    pub allow_airborne: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpatialHints {
    pub p1: ActorHint,
    pub p2: ActorHint,
    /// The first stage confirmed a hit/block contact at or near this frame
    /// (hitstop). Bright, saturated motion regions may then be read as the
    /// contact spark instead of being ignored as noise. Set this only from
    /// meter/HP contact evidence; stage effects can otherwise be misread.
    pub contact_effect: bool,
    /// The frame is close enough to a round start that the players cannot
    /// have crossed sides yet. Identity signatures are learned only here.
    pub sides_certain: bool,
}
