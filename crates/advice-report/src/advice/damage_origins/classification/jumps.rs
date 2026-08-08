use super::super::candidate::{contact_matches, offer, starts_in, Candidate};
use crate::advice::DamageOrigin;
use crate::match_events::{DamageEvent, EventConfidence, JumpOutcome, MatchEvents};

pub(super) fn offer_own_jump_caught(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    for jump in events.jumps.iter().filter(|jump| {
        jump.side == own
            && jump.round_no == damage.round_no
            && jump.takeoff_confirmed
            && jump.outcome == JumpOutcome::GotHit
    }) {
        let matched = jump
            .contact_frame
            .is_some_and(|frame| contact_matches(damage, frame))
            || (jump.contact_frame.is_none()
                && starts_in(
                    damage,
                    jump.frame
                        .saturating_add(crate::match_events::JUMP_SELF_HIT_MIN),
                    jump.air_end
                        .max(jump.frame + crate::match_events::JUMP_SELF_HIT_WINDOW),
                ));
        if matched {
            let anchor = jump.contact_frame.unwrap_or(jump.frame);
            offer(
                candidate,
                DamageOrigin::OwnJumpCaught,
                if jump.contact_frame.is_some() {
                    EventConfidence::High
                } else {
                    EventConfidence::Medium
                },
                88,
                anchor,
                damage,
            );
        }
    }
}

pub(super) fn offer_opponent_jump_in(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for jump in events.jumps.iter().filter(|jump| {
        jump.side == opponent
            && jump.round_no == damage.round_no
            && jump.takeoff_confirmed
            && jump.outcome == JumpOutcome::LandedHit
    }) {
        let matched = jump
            .contact_frame
            .is_some_and(|frame| contact_matches(damage, frame))
            || (jump.contact_frame.is_none()
                && starts_in(
                    damage,
                    jump.frame
                        .saturating_add(crate::match_events::JUMP_ATTACK_MIN),
                    jump.frame
                        .saturating_add(crate::match_events::JUMP_ATTACK_MAX),
                ));
        if matched {
            let anchor = jump.contact_frame.unwrap_or(jump.frame);
            offer(
                candidate,
                DamageOrigin::OpponentJumpIn,
                if jump.contact_frame.is_some() {
                    EventConfidence::High
                } else {
                    EventConfidence::Medium
                },
                70,
                anchor,
                damage,
            );
        }
    }
}
