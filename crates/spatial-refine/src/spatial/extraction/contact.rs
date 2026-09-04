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
        .filter(|(index, region)| spark_candidate(*index, region, used_regions, config))
        .filter_map(|(_, region)| region.spark_centroid().map(|centroid| (region, centroid)))
        .filter(|(_, centroid)| {
            span.is_none_or(|(left, right)| centroid.x >= left && centroid.x <= right)
        })
        .max_by_key(|(region, _)| (region.spark_cells(), region.energy))
        .map(|(region, centroid)| {
            let fraction = region.spark_cells() as f32 / region.changed_cells.max(1) as f32;
            ContactObservation {
                center: centroid,
                bounds: region.bounds,
                effect_cells: region.spark_cells(),
                confidence: spark_confidence(fraction, region.spark_cells()),
            }
        })
}

/// スパークとして数えられる領域は 2 種類ある。
///
/// 1. 単独のスパーク: 本体が凍結していて、エフェクトだけが動いた領域。
///    トラック未割り当てで、スパーク色の割合が高い。
/// 2. 埋め込みスパーク: 実際の hitstop では本体も揺れるため、スパークが
///    本体のモーション領域へ合体することがある。割合は薄まるが、
///    スパーク色セルが「多く」て「凝集」していれば衣装の明色と区別できる
///    (衣装は体に沿って分散する)。こちらはトラック割り当て済みでもよい。
fn spark_candidate(
    index: usize,
    region: &MotionRegion,
    used_regions: &[usize],
    config: &SpatialConfig,
) -> bool {
    // ヒットの暖色とガードの寒色を合わせてスパークの証拠にする。
    let cells = region.spark_cells();
    if cells >= config.contact_min_effect_cells && !used_regions.contains(&index) {
        let fraction = cells as f32 / region.changed_cells.max(1) as f32;
        if fraction >= config.contact_min_effect_fraction {
            return true;
        }
    }
    cells >= config.contact_embedded_min_cells
        && region
            .spark_spread()
            .is_some_and(|spread| spread <= config.contact_embedded_max_spread)
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

#[cfg(test)]
mod tests {
    use super::super::super::{SpatialPoint, SpatialRect};
    use super::*;

    fn actor(x: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(x, 0.9),
            bounds: SpatialRect::new(x - 0.05, 0.6, x + 0.05, 0.9),
            confidence: 0.72,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    #[test]
    fn actor_span_pads_both_sides_of_the_tracked_interval() {
        let left = actor(0.3);
        let right = actor(0.6);
        let approx = |span: Option<(f32, f32)>, expected: (f32, f32)| {
            let (lo, hi) = span.expect("span");
            assert!((lo - expected.0).abs() < 1e-6 && (hi - expected.1).abs() < 1e-6);
        };
        approx(actor_span(Some(&left), Some(&right), 0.1), (0.2, 0.7));
        // 並び順に依存しない。
        approx(actor_span(Some(&right), Some(&left), 0.1), (0.2, 0.7));
        // 片方でも欠ければ span は作れない。
        assert!(actor_span(Some(&left), None, 0.1).is_none());
        assert!(actor_span(None, Some(&right), 0.1).is_none());
    }

    #[test]
    fn spark_confidence_mixes_purity_and_size_with_a_cap() {
        // 0.35 + 0.35*0.5 + (12/24)*0.20 = 0.625
        assert!((spark_confidence(0.5, 12) - 0.625).abs() < 1e-6);
        // サイズ項は 24 セルで頭打ち。
        assert!((spark_confidence(0.5, 48) - 0.725).abs() < 1e-6);
        // 純度は 1.0 で clamp し、全体は 0.90 で cap する。
        assert!((spark_confidence(2.0, 48) - 0.90).abs() < 1e-6);
        assert!((spark_confidence(0.0, 0) - 0.35).abs() < 1e-6);
    }
}
