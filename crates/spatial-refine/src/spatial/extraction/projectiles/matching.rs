use super::super::super::super::{SpatialConfig, SpatialPoint};
use super::super::relationship::point_distance;
use super::ObjectTrack;

pub(super) fn closest_track(
    tracks: &[ObjectTrack],
    center: SpatialPoint,
    frame_index: u32,
    used: &[bool],
    config: &SpatialConfig,
) -> Option<usize> {
    tracks
        .iter()
        .enumerate()
        .filter(|(index, track)| {
            !used[*index]
                && frame_index > track.last_frame
                && frame_index.saturating_sub(track.last_frame) <= config.projectile_max_track_gap
                && point_distance(center, track.center) <= config.projectile_match_distance
        })
        .min_by(|(_, a), (_, b)| {
            point_distance(center, a.center).total_cmp(&point_distance(center, b.center))
        })
        .map(|(index, _)| index)
}
