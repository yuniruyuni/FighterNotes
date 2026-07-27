use super::candidate::contact_matches;
use crate::frame_data::StrikeKind;
use crate::frame_features::FrameFeatures;
use crate::match_events::{DamageEvent, EventConfidence, MatchEvents};

mod input;
mod startup;

pub(super) const ATTACK_LOOKBACK: usize = 90;

#[derive(Debug, Clone, Copy)]
pub(super) struct StrikeAttribution {
    pub(super) kind: StrikeKind,
    pub(super) confidence: EventConfidence,
}

pub(super) fn strike_attribution(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
    opponent_character: Option<&str>,
) -> Option<StrikeAttribution> {
    let character = opponent_character?;
    let attacker = 3 - own;
    let contact = events
        .contacts
        .iter()
        .filter(|contact| {
            contact.attacker == attacker
                && contact.victim == own
                && contact.round_no == damage.round_no
                && contact.hit
                && !contact.projectile
                && contact_matches(damage, contact.frame)
        })
        .min_by_key(|contact| contact.frame.abs_diff(damage.start_frame))?;
    input::match_strike_input(features, events, attacker, contact, character)
}

#[cfg(test)]
pub(super) fn segment_distance(segment: &crate::match_events::InputSegment, target: u32) -> u32 {
    input::segment_distance(segment, target)
}

#[cfg(test)]
pub(super) fn frame_index(features: &[FrameFeatures], frame: u32) -> Option<usize> {
    startup::frame_index(features, frame)
}
