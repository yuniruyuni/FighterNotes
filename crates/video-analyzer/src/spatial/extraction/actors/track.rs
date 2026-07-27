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
