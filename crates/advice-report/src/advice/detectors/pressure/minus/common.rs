use crate::match_events::{EventConfidence, MatchEvents};

pub(super) fn observed_opportunities(
    events: &MatchEvents,
    own: u8,
    selection_count: usize,
) -> usize {
    let observed = if events.minus_situations.is_empty() {
        events
            .presses_while_minus
            .iter()
            .filter(|event| event.side == own && event.confidence == EventConfidence::High)
            .count()
    } else {
        events
            .minus_situations
            .iter()
            .filter(|event| event.side == own && event.confidence == EventConfidence::High)
            .count()
    };
    observed.max(selection_count)
}
