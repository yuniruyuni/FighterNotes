use super::support::*;

#[test]
fn test_press_while_minus_card() {
    use crate::match_events::{MinusPressEvent, MinusPressOutcome, MinusSituationEvent};
    let mut ev = empty_events();
    ev.presses_while_minus = vec![
        MinusPressEvent {
            side: 1,
            frame: 1000,
            minus_frames: 5,
            pressed: "弱".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::CounterHit,
            drop: 0.12,
            confidence: EventConfidence::High,
            source_contact_frame: 980,
            round_no: 1,
        },
        MinusPressEvent {
            side: 1,
            frame: 2000,
            minus_frames: 4,
            pressed: "弱".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::GotAway,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 1980,
            round_no: 1,
        },
        MinusPressEvent {
            side: 1,
            frame: 2500,
            minus_frames: 3,
            pressed: "中".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::CounterHit,
            drop: 0.2,
            confidence: EventConfidence::High,
            source_contact_frame: 2480,
            round_no: 1,
        },
    ];
    ev.minus_situations = vec![
        MinusSituationEvent {
            side: 1,
            frame: 1000,
            minus_frames: 5,
            fastest_action: Some(DefensiveActionKind::Strike),
            action_frame: Some(1000),
            pressed: "弱".to_string(),
            outcome: Some(MinusPressOutcome::CounterHit),
            drop: 0.12,
            confidence: EventConfidence::High,
            source_contact_frame: 980,
            round_no: 1,
        },
        MinusSituationEvent {
            side: 1,
            frame: 2000,
            minus_frames: 4,
            fastest_action: Some(DefensiveActionKind::Strike),
            action_frame: Some(2000),
            pressed: "弱".to_string(),
            outcome: Some(MinusPressOutcome::GotAway),
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 1980,
            round_no: 1,
        },
        MinusSituationEvent {
            side: 1,
            frame: 2500,
            minus_frames: 3,
            fastest_action: Some(DefensiveActionKind::Strike),
            action_frame: Some(2500),
            pressed: "中".to_string(),
            outcome: Some(MinusPressOutcome::CounterHit),
            drop: 0.2,
            confidence: EventConfidence::High,
            source_contact_frame: 2480,
            round_no: 1,
        },
        MinusSituationEvent {
            side: 1,
            frame: 3000,
            minus_frames: 2,
            fastest_action: None,
            action_frame: None,
            pressed: String::new(),
            outcome: None,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 2980,
            round_no: 1,
        },
    ];
    let report = build_report(&[], &ev, "p1", None);
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "press_while_minus")
        .expect("カードが出るべき");
    assert!(card.severity >= 0.32);
    assert!(
        card.description.contains("4 回中、3 回（75%）"),
        "母数と偏り: {}",
        card.description
    );
    assert!(
        card.evidence.iter().any(|e| e.frame == 1000),
        "CounterHit は必ず証拠に入る"
    );
    assert_eq!(card.evidence.len(), 2, "被弾場面だけを件数として数える");
}
