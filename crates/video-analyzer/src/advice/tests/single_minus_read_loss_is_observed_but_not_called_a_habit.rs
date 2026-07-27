use super::support::*;

#[test]
fn single_minus_read_loss_is_observed_but_not_called_a_habit() {
    use crate::match_events::{MinusPressEvent, MinusPressOutcome, MinusSituationEvent};
    let mut ev = empty_events();
    ev.presses_while_minus = vec![MinusPressEvent {
        side: 1,
        frame: 1000,
        minus_frames: 4,
        pressed: "弱".to_string(),
        action_kind: DefensiveActionKind::Strike,
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.12,
        confidence: EventConfidence::High,
        source_contact_frame: 980,
        round_no: 1,
    }];
    ev.minus_situations = vec![MinusSituationEvent {
        side: 1,
        frame: 1000,
        minus_frames: 4,
        fastest_action: Some(DefensiveActionKind::Strike),
        action_frame: Some(1000),
        pressed: "弱".to_string(),
        outcome: Some(MinusPressOutcome::CounterHit),
        drop: 0.12,
        confidence: EventConfidence::High,
        source_contact_frame: 980,
        round_no: 1,
    }];

    let report = build_report(&[], &ev, "p1", None);
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "press_while_minus")
        .expect("単発の被弾も確認場面として残す");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.evidence.len(), 1);
    assert_invites_user_review(card);
    assert!(report
        .cards
        .iter()
        .all(|card| card.id != "throw_while_minus"));
}
