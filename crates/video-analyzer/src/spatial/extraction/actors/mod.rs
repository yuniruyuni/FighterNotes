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
            if let Some([p1, p2]) = initial_tracks(regions, &candidates, frame_index) {
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
        ActorTrackingResult {
            p1,
            p2,
            used_regions: assignments.into_iter().flatten().collect(),
        }
    }
}
