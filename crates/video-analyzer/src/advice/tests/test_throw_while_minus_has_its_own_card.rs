use super::support::*;

#[test]
fn test_throw_while_minus_has_its_own_card() {
    use crate::match_events::{MinusPressEvent, MinusPressOutcome, MinusSituationEvent};
    let mut ev = empty_events();
    let throw_event = |frame, outcome, drop| MinusPressEvent {
        side: 1,
        frame,
        minus_frames: 1,
        pressed: "投げ".to_string(),
        action_kind: DefensiveActionKind::Throw,
        outcome,
        drop,
        confidence: EventConfidence::High,
        source_contact_frame: frame - 20,
        round_no: 1,
    };
    ev.presses_while_minus = vec![
        throw_event(1000, MinusPressOutcome::CounterHit, 0.1),
        throw_event(2000, MinusPressOutcome::CounterHit, 0.08),
        throw_event(3000, MinusPressOutcome::GotAway, 0.0),
    ];
    ev.minus_situations = ev
        .presses_while_minus
        .iter()
        .map(|event| MinusSituationEvent {
            side: event.side,
            frame: event.frame,
            minus_frames: event.minus_frames,
            fastest_action: Some(event.action_kind),
            action_frame: Some(event.frame),
            pressed: event.pressed.clone(),
            outcome: Some(event.outcome),
            drop: event.drop,
            confidence: event.confidence,
            source_contact_frame: event.source_contact_frame,
            round_no: event.round_no,
        })
        .chain(std::iter::once(MinusSituationEvent {
            side: 1,
            frame: 4000,
            minus_frames: 2,
            fastest_action: None,
            action_frame: None,
            pressed: String::new(),
            outcome: None,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 3980,
            round_no: 1,
        }))
        .collect();
    let report = build_report(&[], &ev, "p1", None);
    assert!(report
        .cards
        .iter()
        .any(|card| card.id == "throw_while_minus"));
    assert!(report
        .cards
        .iter()
        .all(|card| card.id != "press_while_minus"));
}
