use super::super::*;
use super::support::empty_events;
use crate::match_events::{EventConfidence, MatchEvents, WhiffEvent, WhiffOutcome};
use crate::AdviceKind;

fn whiff(frame: u32, outcome: WhiffOutcome, drop: f32) -> WhiffEvent {
    WhiffEvent {
        side: 1,
        frame,
        end_frame: frame + 8,
        outcome,
        drop,
        punished_frame: (outcome == WhiffOutcome::Punished).then_some(frame + 15),
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn events_with(whiffs: Vec<WhiffEvent>) -> MatchEvents {
    MatchEvents {
        whiffs,
        ..empty_events()
    }
}

/// 空振りそのものは間合いを測る手段として正当なので指摘しない。
#[test]
fn unpunished_whiffs_are_not_reported() {
    let events = events_with(vec![
        whiff(100, WhiffOutcome::Unpunished, 0.0),
        whiff(400, WhiffOutcome::Unpunished, 0.0),
        whiff(700, WhiffOutcome::Unpunished, 0.0),
    ]);

    assert!(detect_whiff_punished(&events, 1).is_none());
}

/// 単発の被反撃は読み負けと区別できないため事実確認に留める。
#[test]
fn a_single_punished_whiff_stays_an_observation() {
    let events = events_with(vec![
        whiff(100, WhiffOutcome::Punished, 0.2),
        whiff(400, WhiffOutcome::Unpunished, 0.0),
    ]);

    let card = detect_whiff_punished(&events, 1).expect("card");
    assert_eq!(card.id, "whiff_punished");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.title, "空振りした技の硬直を狩られた場面");
    assert!(card.practice.starts_with("クリップで、間合いを測る意図"));
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].end_frame, Some(108));
}

/// 複数回狩られて初めて、技を置く距離の問題として提示する。
#[test]
fn repeated_punished_whiffs_become_a_diagnosis() {
    let events = events_with(vec![
        whiff(100, WhiffOutcome::Punished, 0.2),
        whiff(400, WhiffOutcome::Punished, 0.15),
        whiff(700, WhiffOutcome::Unpunished, 0.0),
    ]);

    let card = detect_whiff_punished(&events, 1).expect("card");
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.title, "届かない技の硬直を繰り返し狩られている");
    assert!(card.practice.starts_with("クリップで、相手のどの位置"));
    assert_eq!(card.evidence.len(), 2);
    assert!((card.severity - 0.37).abs() < 1e-5);
}

/// 相手の空振りを自分の指摘に混ぜない。
#[test]
fn the_opponents_whiffs_are_not_ours() {
    let mut whiffs = vec![
        whiff(100, WhiffOutcome::Punished, 0.2),
        whiff(400, WhiffOutcome::Punished, 0.1),
    ];
    for event in whiffs.iter_mut() {
        event.side = 2;
    }

    assert!(detect_whiff_punished(&events_with(whiffs), 1).is_none());
}

/// 確度の低い観測は分母にも分子にも入れない。
#[test]
fn low_confidence_whiffs_are_ignored() {
    let mut whiffs = vec![
        whiff(100, WhiffOutcome::Punished, 0.2),
        whiff(400, WhiffOutcome::Punished, 0.1),
    ];
    for event in whiffs.iter_mut() {
        event.confidence = EventConfidence::Medium;
    }

    assert!(detect_whiff_punished(&events_with(whiffs), 1).is_none());
}

/// 説明文の割合は「空振りのうち何回狩られたか」。件数だけでは、
/// 10回中2回と2回中2回が同じに見えてしまう。
#[test]
fn the_description_reports_the_punished_share() {
    let events = events_with(vec![
        whiff(100, WhiffOutcome::Punished, 0.1),
        whiff(400, WhiffOutcome::Punished, 0.1),
        whiff(700, WhiffOutcome::Unpunished, 0.0),
        whiff(1000, WhiffOutcome::Unpunished, 0.0),
    ]);

    let card = detect_whiff_punished(&events, 1).expect("card");

    assert!(card.description.contains("接触しなかった技 4 回"));
    assert!(card.description.contains("2 回（50%）"));
}
