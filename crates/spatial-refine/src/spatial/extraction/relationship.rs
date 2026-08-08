use super::super::{ActorObservation, DistanceBand, HorizontalOrder, SpatialConfig, SpatialPoint};

pub(super) fn between_actors(
    point: SpatialPoint,
    p1: Option<&ActorObservation>,
    p2: Option<&ActorObservation>,
) -> bool {
    let (Some(p1), Some(p2)) = (p1, p2) else {
        return true;
    };
    let left = p1.anchor.x.min(p2.anchor.x) - 0.04;
    let right = p1.anchor.x.max(p2.anchor.x) + 0.04;
    point.x >= left && point.x <= right
}

pub(super) fn spatial_relationship(
    p1: Option<&ActorObservation>,
    p2: Option<&ActorObservation>,
    config: &SpatialConfig,
) -> (Option<f32>, Option<DistanceBand>, Option<HorizontalOrder>) {
    let (Some(p1), Some(p2)) = (p1, p2) else {
        return (None, None, None);
    };
    let signed = p2.anchor.x - p1.anchor.x;
    let distance = signed.abs();
    let band = if distance <= config.overlap_distance {
        DistanceBand::Overlap
    } else if distance <= config.close_distance {
        DistanceBand::Close
    } else if distance <= config.mid_distance {
        DistanceBand::Mid
    } else {
        DistanceBand::Far
    };
    let order = if distance <= config.overlap_distance {
        HorizontalOrder::Overlapping
    } else if signed > 0.0 {
        HorizontalOrder::P1Left
    } else {
        HorizontalOrder::P1Right
    };
    (Some(distance), Some(band), Some(order))
}

pub(super) fn point_distance(a: SpatialPoint, b: SpatialPoint) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
