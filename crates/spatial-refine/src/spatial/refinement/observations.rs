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
