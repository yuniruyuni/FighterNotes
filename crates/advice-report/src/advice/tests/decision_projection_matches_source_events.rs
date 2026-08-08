use super::support::*;
use crate::advice::decisions::{
    collect_decisions, losses, option_bias, selections, DecisionOption, DecisionResult,
    DecisionSituation,
};
use crate::match_events::{
    AdvantageOutcome, AdvantageSituationEvent, DefensiveActionKind, MinusPressEvent,
    MinusPressOutcome, MinusSituationEvent,
};

fn press(frame: u32, kind: DefensiveActionKind, outcome: MinusPressOutcome) -> MinusPressEvent {
    MinusPressEvent {
        side: 1,
        frame,
        minus_frames: 5,
        pressed: "弱".to_string(),
        action_kind: kind,
        outcome,
        drop: if outcome == MinusPressOutcome::CounterHit {
            0.1
        } else {
            0.0
        },
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(20),
        round_no: 1,
    }
}

fn situation(frame: u32, fastest: Option<DefensiveActionKind>) -> MinusSituationEvent {
    MinusSituationEvent {
        side: 1,
        frame,
        minus_frames: 5,
        fastest_action: fastest,
        action_frame: fastest.map(|_| frame),
        pressed: String::new(),
        outcome: None,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(20),
        round_no: 1,
    }
}

/// 射影は既存イベントの読み替えであって再導出ではない。不利フレーム後の
/// 回答と結末が、元イベントを直接読んだ場合と一致することを固定する。
/// ここがずれると、既存カードの判定が黙って変わる。
#[test]
fn the_disadvantage_projection_matches_the_source_events() {
    let mut events = empty_events();
    events.presses_while_minus = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
        ),
        press(400, DefensiveActionKind::Strike, MinusPressOutcome::GotAway),
        press(
            700,
            DefensiveActionKind::Throw,
            MinusPressOutcome::CounterHit,
        ),
    ];

    let decisions = collect_decisions(&events, 1);
    let strikes = selections(
        &decisions,
        DecisionSituation::Disadvantage,
        DecisionOption::Strike,
    );
    let throws = selections(
        &decisions,
        DecisionSituation::Disadvantage,
        DecisionOption::Throw,
    );

    // 元イベントを直接数えた結果と一致する。
    let source_strikes = events
        .presses_while_minus
        .iter()
        .filter(|event| event.action_kind == DefensiveActionKind::Strike)
        .count();
    assert_eq!(strikes.len(), source_strikes);
    assert_eq!(throws.len(), 1);
    assert_eq!(losses(&strikes).len(), 1);
    assert_eq!(losses(&throws).len(), 1);
    assert!((losses(&strikes)[0].drop - 0.1).abs() < 1e-6);
    assert_eq!(strikes[0].frames, 5);
    assert_eq!(strikes[0].pressed, "弱");
}

/// 最速打撃／最速投げ以外の回答は、機会としては残るが選択には数えない。
/// この分母が変わると偏りの判定が変わる。
#[test]
fn other_answers_stay_in_the_denominator_only() {
    let mut events = empty_events();
    events.presses_while_minus = vec![press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    )];
    events.minus_situations = vec![
        situation(100, Some(DefensiveActionKind::Strike)),
        situation(400, None),
        situation(700, None),
    ];

    let decisions = collect_decisions(&events, 1);

    assert_eq!(
        selections(
            &decisions,
            DecisionSituation::Disadvantage,
            DecisionOption::NoAttack,
        )
        .len(),
        2
    );
    // 打撃を選んだ機会は presses 側の1件だけで、situation 側と二重にしない。
    assert_eq!(
        selections(
            &decisions,
            DecisionSituation::Disadvantage,
            DecisionOption::Strike,
        )
        .len(),
        1
    );
}

/// 状況ごとに独立して数える。混ざると偏りが実際より小さく見える。
#[test]
fn situations_are_counted_independently() {
    let mut events = empty_events();
    events.presses_while_minus = vec![press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    )];
    events.advantage_situations = vec![
        AdvantageSituationEvent {
            side: 1,
            frame: 400,
            plus_frames: 4,
            follow_up: None,
            action_frame: None,
            pressed: String::new(),
            outcome: AdvantageOutcome::TurnLost,
            drop: 0.2,
            confidence: EventConfidence::High,
            source_contact_frame: 380,
            round_no: 1,
        },
        AdvantageSituationEvent {
            side: 1,
            frame: 700,
            plus_frames: 4,
            follow_up: None,
            action_frame: None,
            pressed: String::new(),
            outcome: AdvantageOutcome::Reset,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 680,
            round_no: 1,
        },
    ];

    let decisions = collect_decisions(&events, 1);

    let advantage_idle = selections(
        &decisions,
        DecisionSituation::Advantage,
        DecisionOption::NoAttack,
    );
    assert_eq!(advantage_idle.len(), 2);
    assert_eq!(losses(&advantage_idle).len(), 1);
    // 不利側の1件はこちらへ混ざらない。
    assert_eq!(
        selections(
            &decisions,
            DecisionSituation::Advantage,
            DecisionOption::Strike,
        )
        .len(),
        0
    );
    assert_eq!(advantage_idle[0].result, DecisionResult::Lost);
}

/// 偏りは状況ごとに、最も多い回答の割合として出す。
#[test]
fn the_top_option_share_is_reported_per_situation() {
    let mut events = empty_events();
    events.presses_while_minus = vec![
        press(100, DefensiveActionKind::Strike, MinusPressOutcome::GotAway),
        press(400, DefensiveActionKind::Strike, MinusPressOutcome::GotAway),
        press(700, DefensiveActionKind::Strike, MinusPressOutcome::GotAway),
    ];
    events.minus_situations = vec![situation(1000, None)];

    let decisions = collect_decisions(&events, 1);
    let (option, top, total) =
        option_bias(&decisions, DecisionSituation::Disadvantage).expect("bias");

    assert_eq!(option, DecisionOption::Strike);
    assert_eq!(top, 3);
    assert_eq!(total, 4);

    // 機会が無い状況では割合を出さない。
    assert_eq!(option_bias(&decisions, DecisionSituation::Okizeme), None);
}

/// 射影の絞り込みは side と確度の両方で効く。どちらか片方だけになると、
/// 相手の判断や未確定の観測が自分の偏りへ混ざる。
#[test]
fn the_projection_filters_on_both_side_and_confidence() {
    use crate::match_events::{KnockdownEvent, OkizemeOutcome};

    let mut events = empty_events();
    // 相手側・確度高。side で落ちなければならない。
    let mut other_side = press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    );
    other_side.side = 2;
    // 自分側・確度低。confidence で落ちなければならない。
    let mut low = press(
        400,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    );
    low.confidence = EventConfidence::Medium;
    events.presses_while_minus = vec![other_side, low];

    let mut other_situation = situation(700, None);
    other_situation.side = 2;
    let mut low_situation = situation(1000, None);
    low_situation.confidence = EventConfidence::Medium;
    events.minus_situations = vec![other_situation, low_situation];

    let mut other_advantage = AdvantageSituationEvent {
        side: 2,
        frame: 1300,
        plus_frames: 4,
        follow_up: None,
        action_frame: None,
        pressed: String::new(),
        outcome: AdvantageOutcome::Reset,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: 1280,
        round_no: 1,
    };
    let mut low_advantage = other_advantage.clone();
    low_advantage.side = 1;
    low_advantage.frame = 1600;
    low_advantage.confidence = EventConfidence::Medium;
    other_advantage.side = 2;
    events.advantage_situations = vec![other_advantage, low_advantage];

    let knockdown = |attacker: u8, confidence| KnockdownEvent {
        side: 3 - attacker,
        attacker,
        frame: 1900,
        wakeup_frame: 2000,
        setup_frames: 40,
        okizeme: OkizemeOutcome::Neutral,
        confidence,
        round_no: 1,
    };
    events.knockdowns = vec![
        knockdown(2, EventConfidence::High),
        knockdown(1, EventConfidence::Medium),
    ];

    let decisions = collect_decisions(&events, 1);

    assert!(decisions.is_empty());
}
