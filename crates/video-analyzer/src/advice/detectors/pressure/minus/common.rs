use crate::advice::{
    MIN_DECISION_BIAS_LOSSES, MIN_DECISION_BIAS_OPPORTUNITIES, MIN_DECISION_BIAS_PERCENT,
    MIN_DECISION_BIAS_SELECTIONS,
};
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

pub(super) fn is_biased(opportunities: usize, selections: usize, losses: usize) -> bool {
    opportunities >= MIN_DECISION_BIAS_OPPORTUNITIES
        && selections >= MIN_DECISION_BIAS_SELECTIONS
        && losses >= MIN_DECISION_BIAS_LOSSES
        && selections * 100 >= opportunities * MIN_DECISION_BIAS_PERCENT
}
