use super::super::{ActorObservation, SpatialObservation};

pub(super) fn reliable_actor_pair(
    observation: &SpatialObservation,
) -> Option<(&ActorObservation, &ActorObservation)> {
    let p1 = observation.p1.as_ref()?;
    let p2 = observation.p2.as_ref()?;
    (p1.confidence >= 0.45 && p2.confidence >= 0.45 && (p1.observed || p2.observed))
        .then_some((p1, p2))
}

/// 区間の最初と最後の安定した距離を、カメラのズームで同じ縮尺へ戻して
/// 返す。SF6 のカメラは接近でズームインするため、生の screen 距離では
/// 実際の前進が縮んで見える。補正は最初のサンプルの縮尺に揃える。
pub(super) fn zoom_corrected_endpoints(
    observations: &[SpatialObservation],
    samples: &[&SpatialObservation],
) -> Option<(f32, f32)> {
    let first = samples.first()?;
    let last = samples.last()?;
    let first_distance = first.screen_distance?;
    let last_distance = last.screen_distance?;
    // 端点の間の全フレームのズーム比を積む(サンプル外のフレームも含む)。
    // camera::estimate の zoom_ratio は探索範囲とセグメント間隔の構造上
    // 1±0.06 に収まるため、積は常に正で有限になる。
    let zoom: f32 = observations
        .iter()
        .filter(|observation| {
            observation.frame_index > first.frame_index
                && observation.frame_index <= last.frame_index
        })
        .filter_map(|observation| observation.camera.as_ref())
        .map(|camera| camera.zoom_ratio)
        .product();
    Some((first_distance, last_distance / zoom))
}

pub(super) fn stable_distance_samples(
    observations: &[SpatialObservation],
    start_frame: u32,
    end_frame: u32,
) -> Vec<&SpatialObservation> {
    observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= start_frame && observation.frame_index <= end_frame
        })
        .filter(|observation| {
            reliable_actor_pair(observation).is_some() && observation.screen_distance.is_some()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{SpatialPoint, SpatialRect};

    fn actor(confidence: f32, observed: bool) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(0.5, 0.9),
            bounds: SpatialRect::new(0.45, 0.6, 0.55, 0.9),
            confidence,
            observed,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    fn pair(p1: ActorObservation, p2: ActorObservation) -> SpatialObservation {
        SpatialObservation {
            frame_index: 1,
            p1: Some(p1),
            p2: Some(p2),
            screen_distance: Some(0.2),
            distance_band: None,
            horizontal_order: None,
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        }
    }

    /// 両者の信頼度が 0.45 以上(境界ちょうどを含む)で、少なくとも
    /// 片方を直接観測できたときだけ、その対を距離の証拠に使う。
    #[test]
    fn a_reliable_pair_needs_both_confidences_and_one_direct_observation() {
        // 境界ちょうど。直接観測は片方で足りる。
        assert!(reliable_actor_pair(&pair(actor(0.45, true), actor(0.45, false))).is_some());
        assert!(reliable_actor_pair(&pair(actor(0.45, false), actor(0.45, true))).is_some());
        // どちらか一方でも信頼度が境界を割れば使わない(7/16 = 0.4375)。
        assert!(reliable_actor_pair(&pair(actor(0.4375, true), actor(0.72, true))).is_none());
        assert!(reliable_actor_pair(&pair(actor(0.72, true), actor(0.4375, true))).is_none());
        // どちらも carry-forward(直接観測なし)の対は距離の証拠にならない。
        assert!(reliable_actor_pair(&pair(actor(0.72, false), actor(0.72, false))).is_none());
    }
}
