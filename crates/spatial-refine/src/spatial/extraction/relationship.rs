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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::SpatialRect;

    fn actor(x: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(x, 0.9),
            bounds: SpatialRect::new(x - 0.02, 0.5, x + 0.02, 0.9),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    #[test]
    fn relationship_bands_include_each_exact_threshold_and_preserve_order() {
        let config = SpatialConfig {
            overlap_distance: 0.1,
            close_distance: 0.2,
            mid_distance: 0.3,
            ..SpatialConfig::default()
        };
        for (distance, band) in [
            (0.1, DistanceBand::Overlap),
            (0.2, DistanceBand::Close),
            (0.3, DistanceBand::Mid),
            (0.31, DistanceBand::Far),
        ] {
            let p1 = actor(0.0);
            let p2 = actor(distance);
            let (measured, measured_band, order) =
                spatial_relationship(Some(&p1), Some(&p2), &config);
            assert!((measured.unwrap() - distance).abs() < 1e-6);
            assert_eq!(measured_band, Some(band));
            assert_eq!(
                order,
                Some(if band == DistanceBand::Overlap {
                    HorizontalOrder::Overlapping
                } else {
                    HorizontalOrder::P1Left
                })
            );
        }

        let p1 = actor(0.4);
        let p2 = actor(0.1);
        assert_eq!(
            spatial_relationship(Some(&p1), Some(&p2), &config).2,
            Some(HorizontalOrder::P1Right)
        );
        assert_eq!(
            spatial_relationship(None, Some(&p2), &config),
            (None, None, None)
        );
    }

    #[test]
    fn equal_x_uses_the_nonpositive_order_when_overlap_is_disabled() {
        let config = SpatialConfig {
            overlap_distance: -0.1,
            ..SpatialConfig::default()
        };
        let p1 = actor(0.4);
        let p2 = actor(0.4);
        assert_eq!(
            spatial_relationship(Some(&p1), Some(&p2), &config).2,
            Some(HorizontalOrder::P1Right)
        );
    }
}
