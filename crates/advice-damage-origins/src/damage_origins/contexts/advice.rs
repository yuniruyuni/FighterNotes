use super::super::candidate::CONTACT_BACK;
use crate::{AdviceCard, DamageBreakdown, DamageContext};

pub fn apply_advice_contexts(breakdown: &mut DamageBreakdown, cards: &[AdviceCard]) {
    for evidence in cards
        .iter()
        .filter(|card| card.id == "mashing")
        .flat_map(|card| &card.evidence)
    {
        let Some(end_frame) = evidence.end_frame else {
            continue;
        };
        let Some(event) = breakdown
            .events
            .iter_mut()
            .min_by_key(|event| event.end_frame.abs_diff(end_frame))
        else {
            continue;
        };
        if event.end_frame.abs_diff(end_frame) <= CONTACT_BACK {
            event.contexts.push(DamageContext::Mashing);
            event.contexts.sort_unstable();
            event.contexts.dedup();
        }
    }
}
