mod candidate;
mod matching;

use super::super::{ActorObservation, ProjectileCandidate, SpatialConfig, SpatialPoint};
use super::motion::{projectile_candidate, MotionRegion};
use super::relationship::between_actors;
use candidate::{build_candidate, CandidateEvidence};
use matching::closest_track;

#[derive(Clone, Debug)]
struct ObjectTrack {
    id: u32,
    center: SpatialPoint,
    last_frame: u32,
    observations: u32,
}

pub(super) struct ProjectileTracker {
    tracks: Vec<ObjectTrack>,
    next_id: u32,
}

impl Default for ProjectileTracker {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            next_id: 1,
        }
    }
}

impl ProjectileTracker {
    pub(super) fn reset(&mut self) {
        self.tracks.clear();
        self.next_id = 1;
    }

    pub(super) fn observe(
        &mut self,
        frame_index: u32,
        regions: &[MotionRegion],
        used_actor_regions: &[usize],
        actors: [Option<&ActorObservation>; 2],
        config: &SpatialConfig,
    ) -> Vec<ProjectileCandidate> {
        let mut candidates: Vec<(usize, f32)> = regions
            .iter()
            .enumerate()
            .filter(|(index, _)| !used_actor_regions.contains(index))
            .filter(|(_, region)| projectile_candidate(region, config))
            .filter(|(_, region)| {
                let center = region.center();
                !actors
                    .into_iter()
                    .flatten()
                    .any(|actor| (center.x - actor.anchor.x).abs() < config.actor_exclusion_dx)
            })
            .filter(|(_, region)| between_actors(region.center(), actors[0], actors[1]))
            .map(|(index, region)| {
                let size_score = (region.changed_cells as f32
                    / config.projectile_max_changed_cells.max(1) as f32)
                    .sqrt()
                    .min(1.0);
                (index, size_score)
            })
            .collect();
        candidates.sort_by(|(a_index, a_score), (b_index, b_score)| {
            b_score
                .total_cmp(a_score)
                .then_with(|| regions[*b_index].energy.cmp(&regions[*a_index].energy))
        });
        candidates.truncate(config.max_projectile_candidates);

        let mut used_tracks = vec![false; self.tracks.len()];
        let output = candidates
            .into_iter()
            .map(|(region_index, size_score)| {
                let region = &regions[region_index];
                let center = region.center();
                let effect_score = region.effect_cells as f32 / region.changed_cells.max(1) as f32;
                let matched =
                    closest_track(&self.tracks, center, frame_index, &used_tracks, config);
                let (track_id, velocity_x, observations) = if let Some(track_index) = matched {
                    used_tracks[track_index] = true;
                    let track = &mut self.tracks[track_index];
                    let dt = frame_index.saturating_sub(track.last_frame).max(1) as f32;
                    let velocity_x = (center.x - track.center.x) / dt;
                    track.center = center;
                    track.last_frame = frame_index;
                    track.observations += 1;
                    (track.id, Some(velocity_x), track.observations)
                } else {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.tracks.push(ObjectTrack {
                        id,
                        center,
                        last_frame: frame_index,
                        observations: 1,
                    });
                    used_tracks.push(true);
                    (id, None, 1)
                };
                build_candidate(
                    region,
                    center,
                    CandidateEvidence {
                        track_id,
                        velocity_x,
                        observations,
                        size_score,
                        effect_score,
                    },
                )
            })
            .collect();
        self.tracks.retain(|track| {
            frame_index.saturating_sub(track.last_frame) <= config.max_stale_frames
        });
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{SpatialConfig, SpatialRect};

    fn region(x: f32) -> MotionRegion {
        MotionRegion {
            bounds: SpatialRect::new(x - 0.03, 0.45, x + 0.03, 0.55),
            changed_cells: 10,
            energy: 1_000,
            effect_cells: 2,
        }
    }

    fn actor(x: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(x, 0.9),
            bounds: SpatialRect::new(x - 0.05, 0.5, x + 0.05, 0.9),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    #[test]
    fn actor_regions_and_actor_proximity_are_independent_exclusions() {
        let regions = [region(0.5)];
        let config = SpatialConfig::default();

        let mut tracker = ProjectileTracker::default();
        assert_eq!(
            tracker.observe(10, &regions, &[0], [None, None], &config),
            []
        );

        let nearby = actor(0.5);
        let mut tracker = ProjectileTracker::default();
        assert_eq!(
            tracker.observe(10, &regions, &[], [Some(&nearby), None], &config),
            []
        );

        let left = actor(0.2);
        let right = actor(0.8);
        let mut tracker = ProjectileTracker::default();
        let candidates = tracker.observe(10, &regions, &[], [Some(&left), Some(&right)], &config);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].center, SpatialPoint::new(0.5, 0.5));
    }

    #[test]
    fn reset_restarts_track_ids() {
        let regions = [region(0.5)];
        let config = SpatialConfig::default();
        let mut tracker = ProjectileTracker::default();
        assert_eq!(
            tracker.observe(10, &regions, &[], [None, None], &config)[0].track_id,
            1
        );
        tracker.reset();
        assert_eq!(
            tracker.observe(20, &regions, &[], [None, None], &config)[0].track_id,
            1
        );
    }
}
