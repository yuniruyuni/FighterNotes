use super::super::candidate::{offer, starts_in, Candidate};
use crate::match_events::{DamageEvent, EventConfidence, MatchEvents, ThrowOutcome};
use crate::DamageOrigin;

const THROW_DAMAGE_WINDOW: u32 = 125;

pub fn offer_throw_actions(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for throw in events.throw_actions.iter().filter(|throw| {
        throw.thrower == opponent
            && throw.round_no == damage.round_no
            && throw.outcome == ThrowOutcome::Hit
            && throw.damage > 0.0
    }) {
        let anchor = throw
            .active_frame
            .or(throw.startup_frame)
            .unwrap_or(throw.input_frame);
        if starts_in(
            damage,
            anchor.saturating_sub(2),
            anchor.saturating_add(THROW_DAMAGE_WINDOW),
        ) {
            offer(
                candidate,
                DamageOrigin::Throw,
                throw.confidence,
                90,
                anchor,
                damage,
            );
        }
    }
}

/// メーター無し解析の互換イベント。入力とHP減少だけなので確度は中とする。
pub fn offer_legacy_throws(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for throw in events.throws.iter().filter(|throw| {
        throw.thrower == opponent && throw.round_no == damage.round_no && throw.connected
    }) {
        if starts_in(
            damage,
            throw.frame,
            throw.frame.saturating_add(THROW_DAMAGE_WINDOW),
        ) {
            offer(
                candidate,
                DamageOrigin::Throw,
                EventConfidence::Medium,
                89,
                throw.frame,
                damage,
            );
        }
    }
}
