use super::super::build_damage_breakdown;
use super::support::{damage, empty_events};
use crate::match_events::{
    BurnoutCause, BurnoutPeriod, DefensiveActionKind, EventConfidence, GuardBreakEvent,
    MinusPressEvent, MinusPressOutcome, PunishChance, PunishOrigin, PunishOutcome,
    PunishReachability, ReversalEvent,
};
use crate::DamageContext;

#[test]
fn event_contexts_are_independent_sorted_and_complete() {
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
        confidence: EventConfidence::High,
        source_contact_frame: 80,
        round_no: 1,
    });
    events.guard_breaks.push(GuardBreakEvent {
        side: 1,
        frame: 105,
        drop: 0.104,
        guard_dir: "L".to_string(),
        broke_to: "N".to_string(),
        round_no: 1,
    });
    events.reversals.push(ReversalEvent {
        side: 1,
        frame: 60,
        drop: 0.1,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.punishes.push(PunishChance {
        frame: 70,
        side: 1,
        advantage: 5,
        outcome: PunishOutcome::WhiffFail,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 70,
        recovery_end_frame: 80,
        source_contact_frame: Some(69),
        attack_start_frame: Some(75),
        attack_active_frame: Some(79),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.1,
        pressed: "弱".to_string(),
        round_no: 1,
    });
    events.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 95,
        end_frame: 110,
        hp_lost: 0.1,
        hp_dealt: 0.0,
        cause: BurnoutCause::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let breakdown = build_damage_breakdown(&[], &events, 1, None);
    assert_eq!(
        breakdown.events[0].contexts,
        [
            DamageContext::PressWhileMinus,
            DamageContext::GuardBreak,
            DamageContext::ReversalPunished,
            DamageContext::PunishWhiff,
            DamageContext::Burnout,
        ]
    );
}
