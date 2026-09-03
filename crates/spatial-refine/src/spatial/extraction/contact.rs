//! Contact-spark localization on hinted contact frames.
//!
//! The first stage already knows *when* a hit or block happened (hitstop
//! stops the frame meter and HP moves). What it cannot know is *where* the
//! attack connected. During hitstop both bodies and the camera freeze, so
//! the frame difference is dominated by the hit effect; the centroid of the
//! bright, saturated changed cells localizes the contact.
//!
//! This module never decides that a contact happened. Without the hint it
//! returns nothing, and stage pyrotechnics outside the actor span are
//! rejected.

use super::super::{ActorObservation, ContactObservation, SpatialConfig};
use super::motion::MotionRegion;

pub(super) fn contact_observation(
    regions: &[MotionRegion],
    used_regions: &[usize],
    p1: Option<&ActorObservation>,
    p2: Option<&ActorObservation>,
    contact_hint: bool,
    config: &SpatialConfig,
) -> Option<ContactObservation> {
    if !contact_hint {
        return None;
    }
    let span = actor_span(p1, p2, config.contact_actor_pad);
    regions
        .iter()
        .enumerate()
        // Saturated costumes also pass the effect-color test, so a region
        // already claimed by an actor track is body motion, not a spark.
        .filter(|(index, _)| !used_regions.contains(index))
        .map(|(_, region)| region)
        .filter(|region| region.effect_cells >= config.contact_min_effect_cells)
        .filter(|region| {
            let fraction = region.effect_cells as f32 / region.changed_cells.max(1) as f32;
            fraction >= config.contact_min_effect_fraction
        })
        .filter_map(|region| region.effect_centroid().map(|centroid| (region, centroid)))
        .filter(|(_, centroid)| {
            span.is_none_or(|(left, right)| centroid.x >= left && centroid.x <= right)
        })
        .max_by_key(|(region, _)| (region.effect_cells, region.energy))
        .map(|(region, centroid)| {
            let fraction = region.effect_cells as f32 / region.changed_cells.max(1) as f32;
            ContactObservation {
                center: centroid,
                bounds: region.bounds,
                effect_cells: region.effect_cells,
                confidence: spark_confidence(fraction, region.effect_cells),
            }
        })
}

/// A spark must appear between the tracked bodies (with slack). Carried
/// anchors are acceptable here: hitstop freezes both actors, so the last
/// known positions are exactly where the bodies are.
fn actor_span(
    p1: Option<&ActorObservation>,
    p2: Option<&ActorObservation>,
    pad: f32,
) -> Option<(f32, f32)> {
    match (p1, p2) {
        (Some(a), Some(b)) => {
            let left = a.anchor.x.min(b.anchor.x) - pad;
            let right = a.anchor.x.max(b.anchor.x) + pad;
            Some((left, right))
        }
        _ => None,
    }
}

/// Deterministic confidence from image evidence alone: how purely the region
/// is effect-colored and how large the effect is.
fn spark_confidence(effect_fraction: f32, effect_cells: u32) -> f32 {
    let size_term = (effect_cells.min(24) as f32 / 24.0) * 0.20;
    (0.35 + 0.35 * effect_fraction.clamp(0.0, 1.0) + size_term).min(0.90)
}
