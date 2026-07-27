use super::super::{ActorObservation, SpatialObservation};

pub(super) fn reliable_actor_pair(
    observation: &SpatialObservation,
) -> Option<(&ActorObservation, &ActorObservation)> {
    let p1 = observation.p1.as_ref()?;
    let p2 = observation.p2.as_ref()?;
    (p1.confidence >= 0.45 && p2.confidence >= 0.45 && (p1.observed || p2.observed))
        .then_some((p1, p2))
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
