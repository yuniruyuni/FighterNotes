mod assignment;
mod track;

use super::super::{ActorObservation, SpatialConfig, SpatialHints};
use super::motion::{actor_candidate, MotionRegion};
use assignment::{assign_regions, initial_tracks};
use track::{apply_anchor_hint, update, ActorTrack};

#[derive(Default)]
pub(super) struct ActorTracker {
    p1: Option<ActorTrack>,
    p2: Option<ActorTrack>,
}

pub(super) struct ActorTrackingResult {
    pub(super) p1: Option<ActorObservation>,
    pub(super) p2: Option<ActorObservation>,
    pub(super) used_regions: Vec<usize>,
}

impl ActorTracker {
    pub(super) fn reset(&mut self) {
        self.p1 = None;
        self.p2 = None;
    }

    pub(super) fn observe(
        &mut self,
        frame_index: u32,
        regions: &[MotionRegion],
        hints: SpatialHints,
        config: &SpatialConfig,
    ) -> ActorTrackingResult {
        apply_anchor_hint(&mut self.p1, hints.p1.anchor, frame_index);
        apply_anchor_hint(&mut self.p2, hints.p2.anchor, frame_index);

        let candidates: Vec<usize> = regions
            .iter()
            .enumerate()
            .filter(|(_, region)| actor_candidate(region, config))
            .map(|(index, _)| index)
            .collect();
        if self.p1.is_none() && self.p2.is_none() && candidates.len() >= 2 {
            if let Some([p1, p2]) = initial_tracks(
                regions,
                &candidates,
                frame_index,
                [hints.p1.allow_airborne, hints.p2.allow_airborne],
                config,
            ) {
                self.p1 = Some(p1);
                self.p2 = Some(p2);
            }
        }

        let assignments = assign_regions(
            [self.p1.as_ref(), self.p2.as_ref()],
            [hints.p1.allow_discontinuity, hints.p2.allow_discontinuity],
            [hints.p1.allow_airborne, hints.p2.allow_airborne],
            regions,
            &candidates,
            config,
        );
        let p1 = update(
            &mut self.p1,
            assignments[0].map(|index| &regions[index]),
            hints.p1.allow_discontinuity,
            frame_index,
            config,
        );
        let p2 = update(
            &mut self.p2,
            assignments[1].map(|index| &regions[index]),
            hints.p2.allow_discontinuity,
            frame_index,
            config,
        );
        let mut used_regions = Vec::new();
        if let Some(index) = assignments[0] {
            used_regions.push(index);
        }
        if let Some(index) = assignments[1] {
            used_regions.push(index);
        }
        ActorTrackingResult {
            p1,
            p2,
            used_regions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::SpatialRect;

    fn region(x: f32) -> MotionRegion {
        MotionRegion {
            bounds: SpatialRect::new(x - 0.05, 0.65, x + 0.05, 0.9),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
        }
    }

    #[test]
    fn first_observation_keeps_frame_identity_and_both_used_regions() {
        let mut tracker = ActorTracker::default();
        let regions = [region(0.25), region(0.75)];

        let result = tracker.observe(
            42,
            &regions,
            SpatialHints::default(),
            &SpatialConfig::default(),
        );

        assert!(result.p1.unwrap().observed);
        assert!(result.p2.unwrap().observed);
        assert_eq!(result.used_regions, [0, 1]);
        assert_eq!(tracker.p1.as_ref().unwrap().last_observed_frame, 42);
        assert_eq!(tracker.p2.as_ref().unwrap().last_observed_frame, 42);
    }

    #[test]
    fn one_existing_track_does_not_reinitialize_the_missing_side() {
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.25), 10, 0.72)),
            p2: None,
        };
        let regions = [region(0.27), region(0.75)];

        let result = tracker.observe(
            11,
            &regions,
            SpatialHints::default(),
            &SpatialConfig::default(),
        );

        assert!(result.p1.is_some());
        assert!(result.p2.is_none());
        assert_eq!(result.used_regions, [0]);
    }

    #[test]
    fn reset_forgets_both_tracks() {
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.25), 10, 0.72)),
            p2: Some(track::from_region(&region(0.75), 10, 0.72)),
        };
        tracker.reset();
        assert!(tracker.p1.is_none());
        assert!(tracker.p2.is_none());
    }

    #[test]
    fn p2_airborne_hint_allows_its_ground_track_to_jump() {
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.25), 10, 0.72)),
            p2: Some(track::from_region(&region(0.75), 10, 0.72)),
        };
        let mut airborne = region(0.74);
        airborne.bounds.top = 0.45;
        airborne.bounds.bottom = 0.70;
        let hints = SpatialHints {
            p2: crate::spatial::ActorHint {
                allow_airborne: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = tracker.observe(
            11,
            &[region(0.26), airborne],
            hints,
            &SpatialConfig::default(),
        );

        assert!(result.p2.unwrap().observed);
    }

    #[test]
    fn p2_discontinuity_hint_reacquires_far_away_and_marks_the_jump() {
        let mut tracker = ActorTracker {
            p1: None,
            p2: Some(track::from_region(&region(0.75), 10, 0.72)),
        };
        let hints = SpatialHints {
            p2: crate::spatial::ActorHint {
                allow_discontinuity: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = tracker.observe(11, &[region(0.40)], hints, &SpatialConfig::default());

        let p2 = result.p2.unwrap();
        assert!(p2.observed);
        assert!(p2.discontinuity);
        assert_eq!(p2.confidence, 0.58);
    }
}
