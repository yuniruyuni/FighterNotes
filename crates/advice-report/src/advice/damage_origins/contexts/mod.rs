mod advice;
mod events;

use crate::advice::{AdviceCard, DamageBreakdown, DamageContext};
use crate::match_events::{DamageEvent, MatchEvents};

pub(super) fn apply_advice_contexts(breakdown: &mut DamageBreakdown, cards: &[AdviceCard]) {
    advice::apply_advice_contexts(breakdown, cards);
}

pub(super) fn damage_contexts(
    match_events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) -> Vec<DamageContext> {
    events::damage_contexts(match_events, own, damage)
}
