mod analysis;
mod card;
mod model;

use crate::match_events::{MatchEvents, PunishOutcome};
use crate::AdviceCard;

pub fn detect_low_conversion(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    if events.contacts.is_empty() {
        return None;
    }
    let successes: Vec<_> = events
        .punishes
        .iter()
        .filter(|punish| punish.side == own && punish.outcome == PunishOutcome::Success)
        .collect();
    let lows: Vec<_> = successes
        .iter()
        .filter_map(|punish| analysis::low_return(events, own, punish))
        .collect();
    if lows.is_empty() {
        return None;
    }
    Some(card::build(successes.len(), &lows))
}
