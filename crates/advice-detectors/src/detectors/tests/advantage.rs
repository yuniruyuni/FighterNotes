use super::super::*;
use super::support::empty_events;
use crate::match_events::{
    AdvantageOutcome, AdvantageSituationEvent, EventConfidence, MatchEvents, PressureFollowUp,
};
use crate::AdviceKind;

fn abandoned(frame: u32, outcome: AdvantageOutcome, drop: f32) -> AdvantageSituationEvent {
    AdvantageSituationEvent {
        side: 1,
        frame,
        plus_frames: 5,
        follow_up: None,
        action_frame: None,
        pressed: String::new(),
        outcome,
        drop,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(20),
        round_no: 1,
    }
}

fn continued(frame: u32) -> AdvantageSituationEvent {
    AdvantageSituationEvent {
        side: 1,
        frame,
        plus_frames: 5,
        follow_up: Some(PressureFollowUp::Strike),
        action_frame: Some(frame),
        pressed: "弱".to_string(),
        outcome: AdvantageOutcome::Continued,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(20),
        round_no: 1,
    }
}

fn events_with(situations: Vec<AdvantageSituationEvent>) -> MatchEvents {
    MatchEvents {
        advantage_situations: situations,
        ..empty_events()
    }
}

/// 攻めを継続しなかっただけでは指摘しない。位置調整やゲージ回復のために
/// 動かない選択は正当なので、ターンを失った結果を伴う場面だけを扱う。
#[test]
fn abandoning_an_advantage_without_losing_the_turn_is_not_reported() {
    let events = events_with(vec![
        abandoned(100, AdvantageOutcome::Reset, 0.0),
        abandoned(400, AdvantageOutcome::Reset, 0.0),
        abandoned(700, AdvantageOutcome::Reset, 0.0),
        abandoned(900, AdvantageOutcome::Reset, 0.0),
    ]);

    assert!(detect_advantage_abandoned(&events, 1).is_none());
}

/// 単発でターンを渡しただけの場面は、癖と断定せず事実確認に留める。
#[test]
fn a_single_lost_turn_stays_an_observation() {
    let events = events_with(vec![
        abandoned(100, AdvantageOutcome::TurnLost, 0.1),
        continued(400),
        continued(700),
        continued(900),
    ]);

    let card = detect_advantage_abandoned(&events, 1).expect("card");
    assert_eq!(card.id, "advantage_abandoned");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.evidence.len(), 1);
}

/// 機会数・放棄数・損失数・選択率が揃って初めて原因診断へ上げる。
#[test]
fn a_repeated_and_biased_abandonment_becomes_a_diagnosis() {
    let events = events_with(vec![
        abandoned(100, AdvantageOutcome::TurnLost, 0.1),
        abandoned(400, AdvantageOutcome::TurnLost, 0.2),
        abandoned(700, AdvantageOutcome::Reset, 0.0),
        continued(900),
    ]);

    let card = detect_advantage_abandoned(&events, 1).expect("card");
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.evidence.len(), 2);
    assert!((card.severity - 0.32).abs() < 1e-5);
}

/// 相手側の有利フレームを自分の指摘に混ぜない。
#[test]
fn the_opponents_advantage_is_not_counted_as_ours() {
    let mut situations = vec![
        abandoned(100, AdvantageOutcome::TurnLost, 0.1),
        abandoned(400, AdvantageOutcome::TurnLost, 0.2),
    ];
    for situation in situations.iter_mut() {
        situation.side = 2;
    }

    assert!(detect_advantage_abandoned(&events_with(situations), 1).is_none());
}

/// 確度の低い機会は分母にも分子にも入れない。
#[test]
fn low_confidence_situations_are_ignored() {
    let mut situations = vec![
        abandoned(100, AdvantageOutcome::TurnLost, 0.1),
        abandoned(400, AdvantageOutcome::TurnLost, 0.2),
    ];
    for situation in situations.iter_mut() {
        situation.confidence = EventConfidence::Medium;
    }

    assert!(detect_advantage_abandoned(&events_with(situations), 1).is_none());
}
