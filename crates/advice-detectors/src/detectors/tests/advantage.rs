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
    assert!((card.severity - 0.11).abs() < 1e-6);
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

/// 相手を端に追い込んだ区間の終わり方は、攻めの放棄と地続きの話として
/// このカードに添える。確認できた終わりが無ければ何も言わず、自分が
/// 端を背負った区間は「追い込んだ」に数えない。
#[test]
fn corner_endings_are_annotated_when_observed() {
    use crate::match_events::{CornerEnding, CornerSpan};
    let mut events = events_with(vec![abandoned(100, AdvantageOutcome::TurnLost, 0.1)]);
    let plain = detect_advantage_abandoned(&events, 1).expect("提示される");
    assert!(
        !plain.description.contains("画面端"),
        "端の確認が無いのに言及している: {}",
        plain.description
    );

    events.corner_spans = vec![
        CornerSpan {
            side: 2,
            start_frame: 300,
            end_frame: 400,
            ending: CornerEnding::SideSwap,
        },
        CornerSpan {
            side: 2,
            start_frame: 600,
            end_frame: 700,
            ending: CornerEnding::Separated,
        },
        CornerSpan {
            side: 2,
            start_frame: 900,
            end_frame: 950,
            ending: CornerEnding::Unobserved,
        },
        CornerSpan {
            side: 1,
            start_frame: 1_200,
            end_frame: 1_300,
            ending: CornerEnding::SideSwap,
        },
    ];
    let card = detect_advantage_abandoned(&events, 1).expect("提示される");
    assert!(
        card.description
            .contains("1 回は左右の入れ替えを許して、1 回は距離が開いて終わっています"),
        "終わり方の内訳が出ていない: {}",
        card.description
    );

    // 片方の終わり方しか確認できていなければ、その分だけを述べる。
    events
        .corner_spans
        .retain(|span| span.ending == CornerEnding::Separated);
    let only_release = detect_advantage_abandoned(&events, 1).expect("提示される");
    assert!(
        only_release
            .description
            .contains("確認できた範囲で 1 回は距離が開いて終わっています"),
        "{}",
        only_release.description
    );
    assert!(
        !only_release.description.contains("入れ替えを許して"),
        "確認できていない終わり方に言及している: {}",
        only_release.description
    );
}
