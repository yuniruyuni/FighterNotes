use crate::advice::MIN_REPEATED_NEGATIVE_OUTCOMES;
use crate::match_events::{MatchEvents, PunishChance, PunishOutcome, PunishReachability};

pub(super) struct Summary<'a> {
    pub(super) failures: Vec<&'a PunishChance>,
    pub(super) success_count: usize,
    pub(super) hp_lost: f32,
    pub(super) repeated_input: Option<&'a str>,
    pub(super) repeated_input_count: usize,
    pub(super) repeated: bool,
    pub(super) min_advantage: u32,
}

pub(super) fn summarize(events: &MatchEvents, own: u8) -> Option<Summary<'_>> {
    let failures: Vec<_> = events
        .punishes
        .iter()
        .filter(|punish| {
            punish.side == own
                && punish.outcome == PunishOutcome::WhiffFail
                && punish.reachability == PunishReachability::Confirmed
        })
        .collect();
    if failures.is_empty() {
        return None;
    }
    let success_count = events
        .punishes
        .iter()
        .filter(|punish| punish.side == own && punish.outcome == PunishOutcome::Success)
        .count();
    let hp_lost = failures.iter().map(|punish| punish.punished_drop).sum();
    let repeated_input = failures
        .iter()
        .filter(|punish| !punish.pressed.is_empty())
        .map(|punish| punish.pressed.as_str())
        .max_by_key(|candidate| {
            failures
                .iter()
                .filter(|punish| punish.pressed.as_str() == *candidate)
                .count()
        });
    let repeated_input_count = repeated_input
        .map(|input| {
            failures
                .iter()
                .filter(|punish| punish.pressed == input)
                .count()
        })
        .unwrap_or(0);
    let repeated = repeated_input_count >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let min_advantage = failures
        .iter()
        .map(|punish| punish.advantage)
        .min()
        .unwrap_or(0);
    Some(Summary {
        failures,
        success_count,
        hp_lost,
        repeated_input,
        repeated_input_count,
        repeated,
        min_advantage,
    })
}
