mod attribution;
mod card;
mod meter;
mod model;
mod pressure;

use crate::advice::{AdviceCard, BIG_DAMAGE};
use crate::frame_features::FrameFeatures;
use crate::match_events::MatchEvents;

pub(crate) fn detect_mashing(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    own_index: usize,
) -> Option<AdviceCard> {
    let segments = &events.segments[own_index];
    if segments.is_empty() {
        return None;
    }
    let opponent = 3 - own;
    let mut hits = Vec::new();
    for damage in events
        .damage
        .iter()
        .filter(|damage| damage.victim == own && damage.drop >= BIG_DAMAGE)
    {
        if attribution::claimed_by_other_detector(events, own, opponent, damage) {
            continue;
        }
        let Some(press) = attribution::nearest_direct_press(segments, damage) else {
            continue;
        };
        let Some(meter_confirmed) = meter::confirm_execution(events, own_index, press, damage)
        else {
            continue;
        };
        if meter::is_neutral_or_counterplay(events, own, own_index, press, damage) {
            continue;
        }
        if !pressure::is_pressured(features, events, own, own_index, damage) {
            continue;
        }
        hits.push(model::MashHit::new(press, damage, meter_confirmed));
    }
    card::build(hits)
}
