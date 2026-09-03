mod assignment;
mod track;

use super::super::{ActorObservation, SpatialConfig, SpatialHints};
use super::grid::CellGrid;
use super::motion::{actor_candidate, MotionRegion};
use super::shadows::ShadowCandidate;
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
        grid: &CellGrid,
        shadows: &[ShadowCandidate],
        hints: SpatialHints,
        config: &SpatialConfig,
    ) -> ActorTrackingResult {
        apply_anchor_hint(&mut self.p1, hints.p1.anchor, frame_index);
        apply_anchor_hint(&mut self.p2, hints.p2.anchor, frame_index);

        let candidates: Vec<usize> = regions
            .iter()
            .enumerate()
            .filter(|(_, region)| actor_candidate(region, config))
            // Hitstop freezes both bodies, so on a hinted contact frame a
            // strongly effect-colored region is the spark, not a player.
            // Without this gate the spark captures a frozen track: the meter
            // overlay exclusion keeps mid-screen anchors above the ground
            // band, which disarms the leaves-ground check.
            .filter(|(_, region)| {
                let effect_fraction =
                    region.effect_cells as f32 / region.changed_cells.max(1) as f32;
                !(hints.contact_effect
                    && effect_fraction >= config.contact_effect_max_actor_fraction)
            })
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
        let merged_with_both = |index: Option<usize>| -> bool {
            let (Some(index), Some(own), Some(other)) = (index, self.p1.as_ref(), self.p2.as_ref())
            else {
                return false;
            };
            let bounds = regions[index].bounds;
            bounds.width() >= config.merged_region_min_width
                && (bounds.left..=bounds.right).contains(&own.anchor.x)
                && (bounds.left..=bounds.right).contains(&other.anchor.x)
        };
        let p1_merged = merged_with_both(assignments[0]);
        let p2_merged = merged_with_both(assignments[1]);
        let p1 = update(
            &mut self.p1,
            assignments[0].map(|index| &regions[index]),
            p1_merged,
            grid,
            hints.p1.allow_discontinuity,
            frame_index,
            config,
        );
        let p2 = update(
            &mut self.p2,
            assignments[1].map(|index| &regions[index]),
            p2_merged,
            grid,
            hints.p2.allow_discontinuity,
            frame_index,
            config,
        );
        let p1 = snap_to_shadow(&mut self.p1, p1, shadows, config);
        let p2 = snap_to_shadow(&mut self.p2, p2, shadows, config);
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

/// 接地影は静止・領域マージ・overlay 除外の影響を受けない毎フレームの
/// 水平証拠なので、近くに影クラスタがあれば anchor.x をその重心へ寄せる。
/// 空中でも影は真下に残るため、x の補正としてはジャンプ中も正しい。
fn snap_to_shadow(
    track: &mut Option<ActorTrack>,
    observation: Option<ActorObservation>,
    shadows: &[ShadowCandidate],
    config: &SpatialConfig,
) -> Option<ActorObservation> {
    let mut observation = observation?;
    let nearest = shadows.iter().min_by(|a, b| {
        (a.center_x - observation.anchor.x)
            .abs()
            .total_cmp(&(b.center_x - observation.anchor.x).abs())
    });
    if let Some(nearest) = nearest {
        if (nearest.center_x - observation.anchor.x).abs() <= config.shadow_snap_dx {
            observation.anchor.x = nearest.center_x;
            if let Some(track) = track.as_mut() {
                track.anchor.x = nearest.center_x;
            }
        }
    }
    Some(observation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::SpatialRect;

    fn empty_grid() -> CellGrid {
        CellGrid {
            cells: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    fn region(x: f32) -> MotionRegion {
        MotionRegion {
            bounds: SpatialRect::new(x - 0.05, 0.65, x + 0.05, 0.9),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
        }
    }

    #[test]
    fn first_observation_keeps_frame_identity_and_both_used_regions() {
        let mut tracker = ActorTracker::default();
        let regions = [region(0.25), region(0.75)];

        let result = tracker.observe(
            42,
            &regions,
            &empty_grid(),
            &[],
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
            &empty_grid(),
            &[],
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
    fn shadow_snapping_targets_the_nearest_cluster_within_reach() {
        let config = SpatialConfig::default();
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.31), 10, 0.72)),
            p2: None,
        };
        // 近い方(軽い 0.335)を選ぶ。重い 0.28 ではない。
        let shadows = [
            ShadowCandidate {
                center_x: 0.28,
                weight: 100.0,
            },
            ShadowCandidate {
                center_x: 0.335,
                weight: 5.0,
            },
        ];
        let result = tracker.observe(
            11,
            &[region(0.31)],
            &empty_grid(),
            &shadows,
            SpatialHints::default(),
            &config,
        );
        assert_eq!(result.p1.unwrap().anchor.x, 0.335);
        // スナップはトラック本体にも残り、観測が絶えても持ち越される。
        let carried = tracker.observe(
            12,
            &[],
            &empty_grid(),
            &[],
            SpatialHints::default(),
            &config,
        );
        assert_eq!(carried.p1.unwrap().anchor.x, 0.335);
    }

    #[test]
    fn shadow_snapping_reach_is_inclusive_and_bounded() {
        let config = SpatialConfig::default();
        // 距離ちょうど snap_dx (0.06) は吸着する。f32 で正確に表すため
        // anchor 0.0 と影 0.06 を使う。
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.0), 10, 0.72)),
            p2: None,
        };
        let at_reach = [ShadowCandidate {
            center_x: 0.06,
            weight: 10.0,
        }];
        let result = tracker.observe(
            11,
            &[region(0.0)],
            &empty_grid(),
            &at_reach,
            SpatialHints::default(),
            &config,
        );
        assert_eq!(result.p1.unwrap().anchor.x, 0.06);

        // 届かない影には吸着しない。
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.30), 10, 0.72)),
            p2: None,
        };
        let out_of_reach = [ShadowCandidate {
            center_x: 0.40,
            weight: 10.0,
        }];
        let result = tracker.observe(
            11,
            &[region(0.30)],
            &empty_grid(),
            &out_of_reach,
            SpatialHints::default(),
            &config,
        );
        assert_eq!(result.p1.unwrap().anchor.x, 0.30);
    }

    #[test]
    fn anchor_hints_carry_the_frame_identity_through_decay() {
        let config = SpatialConfig::default();
        let mut tracker = ActorTracker::default();
        let hints = SpatialHints {
            p1: crate::spatial::ActorHint {
                anchor: Some(crate::spatial::SpatialPoint::new(0.3, 0.9)),
                ..Default::default()
            },
            ..Default::default()
        };
        tracker.observe(100, &[], &empty_grid(), &[], hints, &config);
        let result = tracker.observe(
            102,
            &[],
            &empty_grid(),
            &[],
            SpatialHints::default(),
            &config,
        );
        let p1 = result.p1.unwrap();
        assert!(
            (p1.confidence - 0.80 * 0.92f32.powi(2)).abs() < 1e-6,
            "減衰はヒントを置いたフレームから数える: {p1:?}"
        );
    }

    #[test]
    fn p1_airborne_hint_allows_its_ground_track_to_jump() {
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.25), 10, 0.72)),
            p2: Some(track::from_region(&region(0.75), 10, 0.72)),
        };
        let mut airborne = region(0.26);
        airborne.bounds.top = 0.45;
        airborne.bounds.bottom = 0.62;
        let hints = SpatialHints {
            p1: crate::spatial::ActorHint {
                allow_airborne: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = tracker.observe(
            11,
            &[airborne, region(0.74)],
            &empty_grid(),
            &[],
            hints,
            &SpatialConfig::default(),
        );

        assert!(result.p1.unwrap().observed);
    }

    #[test]
    fn p2_airborne_hint_allows_its_ground_track_to_jump() {
        let mut tracker = ActorTracker {
            p1: Some(track::from_region(&region(0.25), 10, 0.72)),
            p2: Some(track::from_region(&region(0.75), 10, 0.72)),
        };
        let mut airborne = region(0.74);
        airborne.bounds.top = 0.45;
        airborne.bounds.bottom = 0.62;
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
            &empty_grid(),
            &[],
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

        let result = tracker.observe(
            11,
            &[region(0.40)],
            &empty_grid(),
            &[],
            hints,
            &SpatialConfig::default(),
        );

        let p2 = result.p2.unwrap();
        assert!(p2.observed);
        assert!(p2.discontinuity);
        assert_eq!(p2.confidence, 0.58);
    }
}
