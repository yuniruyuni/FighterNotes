use super::support::*;

#[test]
fn cause_specific_observation_owns_the_same_big_damage() {
    use crate::match_events::{MinusPressEvent, MinusPressOutcome, MinusSituationEvent};

    let mut ev = empty_events();
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1020,
        pre_freeze_frame: 1018,
        end_frame: 1080,
        hp_before: 1.0,
        hp_after: 0.6,
        drop: 0.4,
        round_no: 1,
    });
    ev.presses_while_minus.push(MinusPressEvent {
        side: 1,
        frame: 1000,
        minus_frames: 4,
        pressed: "弱".to_string(),
        action_kind: DefensiveActionKind::Strike,
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.4,
        confidence: EventConfidence::High,
        source_contact_frame: 980,
        round_no: 1,
    });
    ev.minus_situations.push(MinusSituationEvent {
        side: 1,
        frame: 1000,
        minus_frames: 4,
        fastest_action: Some(DefensiveActionKind::Strike),
        action_frame: Some(1000),
        pressed: "弱".to_string(),
        outcome: Some(MinusPressOutcome::CounterHit),
        drop: 0.4,
        confidence: EventConfidence::High,
        source_contact_frame: 980,
        round_no: 1,
    });

    let report = detector_test_report(&ev, "p1");
    assert!(report
        .cards
        .iter()
        .any(|card| { card.id == "press_while_minus" && card.kind == AdviceKind::Observation }));
    assert!(report.cards.iter().all(|card| card.id != "big_hits"));
}
