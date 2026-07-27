use super::candidate::Candidate;
use crate::match_events::{DamageEvent, MatchEvents};

mod contacts;
mod drive;
mod jumps;
mod threats;
mod throws;

pub(super) fn classify_damage(events: &MatchEvents, own: u8, damage: &DamageEvent) -> Candidate {
    let mut candidate = None;

    threats::offer_compound(&mut candidate, events, own, damage);
    threats::offer_teleport(&mut candidate, events, own, damage);
    throws::offer_throw_actions(&mut candidate, events, own, damage);
    throws::offer_legacy_throws(&mut candidate, events, own, damage);
    drive::offer_drive_impacts(&mut candidate, events, own, damage);
    drive::offer_drive_rushes(&mut candidate, events, own, damage);
    jumps::offer_own_jump_caught(&mut candidate, events, own, damage);
    contacts::offer_projectile_contacts(&mut candidate, events, own, damage);
    contacts::offer_projectile_threats(&mut candidate, events, own, damage);
    jumps::offer_opponent_jump_in(&mut candidate, events, own, damage);
    contacts::offer_strike_contacts(&mut candidate, events, own, damage);

    candidate.unwrap_or_else(Candidate::unclassified)
}
