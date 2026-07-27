mod analysis;
mod card;

use super::options::failed_option_text;
use crate::advice::AdviceCard;
use crate::match_events::MatchEvents;

pub(crate) fn detect_punish_fail(
    events: &MatchEvents,
    own: u8,
    own_character: Option<&str>,
) -> Option<AdviceCard> {
    let summary = analysis::summarize(events, own)?;
    let option_text = failed_option_text(own_character, summary.min_advantage);
    Some(card::build(&summary, &option_text))
}
