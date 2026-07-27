use super::model::{SpatialCandidateWindow, SpatialHintRange};
use super::round_bounds;
use crate::match_events::{CompoundThreat, JumpEvent, RoundInfo, TeleportEvent};

pub(super) fn windows(
    teleports: &[TeleportEvent],
    compound_threats: &[CompoundThreat],
    jumps: &[JumpEvent],
    rounds: &[RoundInfo],
) -> Vec<SpatialCandidateWindow> {
    teleports
        .iter()
        .map(|teleport| {
            let compound_start = compound_threats
                .iter()
                .find(|threat| {
                    threat.attacker == teleport.attacker
                        && threat.teleport_frame == teleport.input_frame
                })
                .map(|threat| threat.projectile_start_frame.saturating_sub(5));
            let bounds = round_bounds::for_round(rounds, teleport.round_no);
            let start_frame = compound_start
                .unwrap_or_else(|| teleport.input_frame.saturating_sub(20))
                .max(bounds.start);
            let semantic_end = teleport
                .followup_contact_frame
                .or(teleport.followup_attack_frame)
                .unwrap_or(teleport.inv_end_frame);
            let end_frame = semantic_end.saturating_add(25).min(bounds.end);
            let airborne_hints = jumps
                .iter()
                .filter(|jump| {
                    jump.side == teleport.attacker
                        && jump.air_end >= start_frame
                        && jump.frame <= end_frame
                })
                .map(|jump| SpatialHintRange {
                    side: jump.side,
                    start_frame: jump.frame,
                    end_frame: jump.air_end,
                })
                .collect();
            SpatialCandidateWindow {
                start_frame,
                end_frame,
                teleport_hints: vec![SpatialHintRange {
                    side: teleport.attacker,
                    start_frame: teleport.inv_start_frame.saturating_sub(2),
                    end_frame: teleport.inv_end_frame.saturating_add(4),
                }],
                airborne_hints,
            }
        })
        .collect()
}
