use super::super::super::super::{HorizontalMotion, ProjectileCandidate, SpatialPoint};
use super::super::motion::MotionRegion;

pub(super) struct CandidateEvidence {
    pub(super) track_id: u32,
    pub(super) velocity_x: Option<f32>,
    pub(super) observations: u32,
    pub(super) size_score: f32,
    pub(super) effect_score: f32,
}

pub(super) fn build_candidate(
    region: &MotionRegion,
    center: SpatialPoint,
    evidence: CandidateEvidence,
) -> ProjectileCandidate {
    let motion = match evidence.velocity_x {
        Some(value) if value < -0.001 => HorizontalMotion::Left,
        Some(value) if value > 0.001 => HorizontalMotion::Right,
        Some(_) => HorizontalMotion::Stationary,
        None => HorizontalMotion::Unknown,
    };
    let trajectory_confirmed = evidence.observations >= 2
        && !matches!(
            motion,
            HorizontalMotion::Unknown | HorizontalMotion::Stationary
        );
    let confidence = if trajectory_confirmed {
        (0.52 + evidence.size_score * 0.22 + evidence.effect_score * 0.22).min(0.93)
    } else {
        (0.20 + evidence.size_score * 0.24 + evidence.effect_score * 0.18).min(0.58)
    };
    ProjectileCandidate {
        track_id: evidence.track_id,
        center,
        bounds: region.bounds,
        velocity_x: evidence.velocity_x,
        motion,
        trajectory_confirmed,
        confidence,
    }
}
