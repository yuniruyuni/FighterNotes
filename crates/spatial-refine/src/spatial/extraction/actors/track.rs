use super::super::super::{ActorObservation, SpatialConfig, SpatialPoint, SpatialRect};
use super::super::motion::MotionRegion;

#[derive(Clone, Debug)]
pub(super) struct ActorTrack {
    pub(super) anchor: SpatialPoint,
    pub(super) bounds: SpatialRect,
    pub(super) confidence: f32,
    pub(super) last_observed_frame: u32,
}

pub(super) fn apply_anchor_hint(
    track: &mut Option<ActorTrack>,
    hint: Option<SpatialPoint>,
    frame_index: u32,
) {
    let Some(anchor) = hint else {
        return;
    };
    let anchor = SpatialPoint::new(anchor.x.clamp(0.0, 1.0), anchor.y.clamp(0.0, 1.0));
    match track {
        Some(track) => {
            track.anchor = anchor;
            track.bounds = SpatialRect::new(anchor.x, anchor.y, anchor.x, anchor.y);
            track.confidence = track.confidence.max(0.80);
            track.last_observed_frame = frame_index;
        }
        None => {
            *track = Some(ActorTrack {
                anchor,
                bounds: SpatialRect::new(anchor.x, anchor.y, anchor.x, anchor.y),
                confidence: 0.80,
                last_observed_frame: frame_index,
            });
        }
    }
}

pub(super) fn update(
    track: &mut Option<ActorTrack>,
    region: Option<&MotionRegion>,
    allow_discontinuity: bool,
    frame_index: u32,
    config: &SpatialConfig,
) -> Option<ActorObservation> {
    if let Some(region) = region {
        let anchor = region.anchor();
        let discontinuity = track.as_ref().is_some_and(|old| {
            allow_discontinuity && (old.anchor.x - anchor.x).abs() > config.max_track_dx
        });
        let confidence = if discontinuity { 0.58 } else { 0.72 };
        *track = Some(from_region(region, frame_index, confidence));
        return Some(ActorObservation {
            anchor,
            bounds: region.bounds,
            confidence,
            observed: true,
            ground_anchor: anchor.y >= config.actor_ground_y,
            discontinuity,
        });
    }

    let track = track.as_mut()?;
    let stale = frame_index.saturating_sub(track.last_observed_frame);
    if stale > config.max_stale_frames {
        return None;
    }
    let confidence = track.confidence * 0.92f32.powi(stale as i32);
    Some(ActorObservation {
        anchor: track.anchor,
        bounds: track.bounds,
        confidence,
        observed: false,
        ground_anchor: track.anchor.y >= config.actor_ground_y,
        discontinuity: false,
    })
}

pub(super) fn from_region(region: &MotionRegion, frame_index: u32, confidence: f32) -> ActorTrack {
    ActorTrack {
        anchor: region.anchor(),
        bounds: region.bounds,
        confidence,
        last_observed_frame: frame_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: f32, y: f32) -> MotionRegion {
        MotionRegion {
            bounds: SpatialRect::new(x - 0.05, y - 0.2, x + 0.05, y),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
        }
    }

    #[test]
    fn anchor_hints_clamp_coordinates_and_update_every_track_field() {
        let mut track = None;
        apply_anchor_hint(&mut track, Some(SpatialPoint::new(-0.2, 1.4)), 37);
        let created = track.as_ref().unwrap();
        assert_eq!(created.anchor, SpatialPoint::new(0.0, 1.0));
        assert_eq!(created.bounds, SpatialRect::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(created.confidence, 0.80);
        assert_eq!(created.last_observed_frame, 37);

        track.as_mut().unwrap().confidence = 0.91;
        apply_anchor_hint(&mut track, Some(SpatialPoint::new(0.4, 0.6)), 42);
        let updated = track.as_ref().unwrap();
        assert_eq!(updated.anchor, SpatialPoint::new(0.4, 0.6));
        assert_eq!(updated.bounds, SpatialRect::new(0.4, 0.6, 0.4, 0.6));
        assert_eq!(updated.confidence, 0.91);
        assert_eq!(updated.last_observed_frame, 42);

        let snapshot = track.clone();
        apply_anchor_hint(&mut track, None, 99);
        assert_eq!(
            track.as_ref().unwrap().last_observed_frame,
            snapshot.unwrap().last_observed_frame
        );
    }

    #[test]
    fn region_updates_report_discontinuity_observation_and_ground_boundary() {
        let config = SpatialConfig::default();
        let mut track = Some(from_region(&region(0.1, config.actor_ground_y), 5, 0.72));
        let moved = region(0.1 + config.max_track_dx + 0.01, config.actor_ground_y);

        let observed = update(&mut track, Some(&moved), true, 17, &config).unwrap();

        assert_eq!(observed.anchor, moved.anchor());
        assert_eq!(observed.bounds, moved.bounds);
        assert_eq!(observed.confidence, 0.58);
        assert!(observed.observed);
        assert!(observed.ground_anchor);
        assert!(observed.discontinuity);
        let stored = track.unwrap();
        assert_eq!(stored.last_observed_frame, 17);
        assert_eq!(stored.confidence, 0.58);
    }

    #[test]
    fn stale_tracks_decay_through_the_last_allowed_frame() {
        let config = SpatialConfig {
            max_stale_frames: 2,
            ..SpatialConfig::default()
        };
        let mut track = Some(from_region(&region(0.3, 0.7), 10, 0.8));

        let stale = update(&mut track, None, false, 12, &config).unwrap();
        assert!((stale.confidence - 0.8 * 0.92f32.powi(2)).abs() < 1e-6);
        assert!(!stale.observed);
        assert!(!stale.ground_anchor);
        assert!(!stale.discontinuity);
        assert!(update(&mut track, None, false, 13, &config).is_none());

        let stored = from_region(&region(0.2, 0.9), 23, 0.67);
        assert_eq!(stored.last_observed_frame, 23);
        assert_eq!(stored.confidence, 0.67);

        let mut grounded = Some(from_region(&region(0.2, config.actor_ground_y), 20, 0.7));
        assert!(
            update(&mut grounded, None, false, 20, &config)
                .unwrap()
                .ground_anchor
        );
    }
}
