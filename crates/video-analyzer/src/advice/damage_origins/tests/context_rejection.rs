use super::super::build_damage_breakdown;
use super::support::{damage, empty_events};
use crate::match_events::{
    BurnoutCause, BurnoutPeriod, DefensiveActionKind, EventConfidence, GuardBreakEvent,
    MinusPressEvent, MinusPressOutcome, ReversalEvent,
};

#[test]
fn event_contexts_reject_low_confidence_or_mismatched_evidence() {
    let mut events = empty_events();
    events.damage.push(damage(100, 1, 0.1));
    events.presses_while_minus.push(MinusPressEvent {
        side: 1,
        frame: 90,
        minus_frames: 4,
        pressed: "弱".to_string(),
        action_kind: DefensiveActionKind::Strike,
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.1,
        confidence: EventConfidence::Low,
        source_contact_frame: 80,
        round_no: 1,
    });
    events.guard_breaks.push(GuardBreakEvent {
        side: 1,
        frame: 105,
        drop: 0.2,
        guard_dir: "L".to_string(),
        broke_to: "N".to_string(),
        round_no: 1,
    });
    events.reversals.push(ReversalEvent {
        side: 2,
        frame: 60,
        drop: 0.1,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 200,
        end_frame: 250,
        hp_lost: 0.1,
        hp_dealt: 0.0,
        cause: BurnoutCause::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert!(breakdown.events[0].contexts.is_empty());
}
