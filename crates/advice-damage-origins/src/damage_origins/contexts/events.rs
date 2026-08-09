use super::super::candidate::{approximately_same_drop, starts_in, CONTACT_BACK};
use crate::match_events::{
    DamageEvent, EventConfidence, MatchEvents, MinusPressOutcome, PunishOutcome,
};
use crate::DamageContext;

const REVERSAL_RESULT_WINDOW: u32 = 105;
const PUNISH_RESULT_WINDOW: u32 = 45;

pub fn damage_contexts(events: &MatchEvents, own: u8, damage: &DamageEvent) -> Vec<DamageContext> {
    let mut contexts = Vec::new();

    if events.presses_while_minus.iter().any(|press| {
        press.side == own
            && press.round_no == damage.round_no
            && press.outcome == MinusPressOutcome::CounterHit
            && press.confidence != EventConfidence::Low
            && starts_in(damage, press.frame, press.frame.saturating_add(30))
    }) {
        contexts.push(DamageContext::PressWhileMinus);
    }
    if events.guard_breaks.iter().any(|guard_break| {
        guard_break.side == own
            && guard_break.round_no == damage.round_no
            && guard_break.frame.abs_diff(damage.start_frame) <= CONTACT_BACK
            && approximately_same_drop(guard_break.drop, damage.drop)
    }) {
        contexts.push(DamageContext::GuardBreak);
    }
    if events.reversals.iter().any(|reversal| {
        reversal.side == own
            && reversal.round_no == damage.round_no
            && starts_in(
                damage,
                reversal.frame,
                reversal.frame.saturating_add(REVERSAL_RESULT_WINDOW),
            )
            && approximately_same_drop(reversal.drop, damage.drop)
    }) {
        contexts.push(DamageContext::ReversalPunished);
    }
    if events.punishes.iter().any(|punish| {
        punish.side == own
            && punish.round_no == damage.round_no
            && punish.outcome == PunishOutcome::WhiffFail
            && punish.punished_drop > 0.0
            && starts_in(
                damage,
                punish.recovery_end_frame,
                punish
                    .recovery_end_frame
                    .saturating_add(PUNISH_RESULT_WINDOW),
            )
            && approximately_same_drop(punish.punished_drop, damage.drop)
    }) {
        contexts.push(DamageContext::PunishWhiff);
    }
    if events.burnouts.iter().any(|burnout| {
        burnout.side == own
            && burnout.round_no == damage.round_no
            && damage.start_frame <= burnout.end_frame
            && damage.end_frame >= burnout.start_frame
    }) {
        contexts.push(DamageContext::Burnout);
    }

    contexts.sort_unstable();
    contexts.dedup();
    contexts
}
