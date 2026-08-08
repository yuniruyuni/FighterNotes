use super::super::{HorizontalMotion, HorizontalOrder, SpatialObservation};
use crate::match_events::{CompoundThreat, ProjectileThreat};

pub(super) fn refine(projectiles: &mut [ProjectileThreat], observations: &[SpatialObservation]) {
    for projectile in projectiles {
        let end_frame = projectile
            .contact_frame
            .unwrap_or(projectile.observed_end_frame);
        let trajectory_seen = observations.iter().any(|observation| {
            observation.frame_index >= projectile.observed_start_frame
                && observation.frame_index <= end_frame
                && observation.projectile_candidates.iter().any(|candidate| {
                    candidate.trajectory_confirmed
                        && candidate.confidence >= 0.65
                        && motion_matches(
                            projectile.owner,
                            observation.horizontal_order,
                            candidate.motion,
                        )
                })
        });
        if trajectory_seen {
            projectile.confidence = projectile.confidence.max(0.95);
        }
    }
}

pub(super) fn propagate_confidence(
    projectiles: &[ProjectileThreat],
    threats: &mut [CompoundThreat],
) {
    for threat in threats {
        if let Some(projectile) = projectiles.iter().find(|projectile| {
            projectile.owner == threat.attacker
                && projectile.observed_start_frame == threat.projectile_start_frame
        }) {
            threat.confidence = threat.confidence.max(projectile.confidence.min(0.95));
        }
    }
}

fn motion_matches(owner: u8, order: Option<HorizontalOrder>, motion: HorizontalMotion) -> bool {
    matches!(
        (owner, order, motion),
        (1, Some(HorizontalOrder::P1Left), HorizontalMotion::Right)
            | (1, Some(HorizontalOrder::P1Right), HorizontalMotion::Left)
            | (2, Some(HorizontalOrder::P1Left), HorizontalMotion::Left)
            | (2, Some(HorizontalOrder::P1Right), HorizontalMotion::Right)
    )
}
