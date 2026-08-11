//! 状況ラベル × 選択肢への射影に対するテスト。
//!
//! ここは射影であって再導出ではない。イベント層が出した結論を、状況・回答・
//! 結末という同じ形へ並べ替えるだけであることを固定する。並べ替えの過程で
//! 結論が変わってしまうと、偏りの判定が実態とずれる。

use super::*;
use crate::match_events::{
    AdvantageOutcome, AdvantageSituationEvent, DefensiveActionKind, KnockdownEvent,
    MinusPressEvent, MinusSituationEvent, OkizemeOutcome,
};
use match_event_layer::test_support::empty_events;

fn minus_press(
    frame: u32,
    kind: DefensiveActionKind,
    outcome: MinusPressOutcome,
) -> MinusPressEvent {
    MinusPressEvent {
        side: 1,
        frame,
        minus_frames: 5,
        pressed: "LP".to_string(),
        action_kind: kind,
        outcome,
        drop: 0.1,
        confidence: EventConfidence::High,
        source_contact_frame: frame,
        round_no: 1,
    }
}

fn advantage(
    frame: u32,
    action_frame: Option<u32>,
    outcome: AdvantageOutcome,
) -> AdvantageSituationEvent {
    AdvantageSituationEvent {
        side: 1,
        frame,
        plus_frames: 4,
        follow_up: None,
        action_frame,
        pressed: String::new(),
        outcome,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: frame,
        round_no: 1,
    }
}

fn knockdown(wakeup_frame: u32, okizeme: OkizemeOutcome) -> KnockdownEvent {
    KnockdownEvent {
        side: 2,
        attacker: 1,
        frame: wakeup_frame - 60,
        wakeup_frame,
        setup_frames: 30,
        okizeme,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 不利フレームからの回答は、打撃と投げを取り違えずに写す。
/// 結末は「反撃を食らった」だけを負けとする。
#[test]
fn a_challenge_while_minus_keeps_its_action_and_result() {
    let mut events = empty_events();
    events.presses_while_minus.push(minus_press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    ));
    events.presses_while_minus.push(minus_press(
        200,
        DefensiveActionKind::Throw,
        MinusPressOutcome::Won,
    ));
    events.presses_while_minus.push(minus_press(
        300,
        DefensiveActionKind::Throw,
        MinusPressOutcome::GotAway,
    ));

    let decisions = collect_decisions(&events, 1);

    let kinds: Vec<_> = decisions
        .iter()
        .map(|event| (event.situation, event.option, event.result))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (
                DecisionSituation::Disadvantage,
                DecisionOption::Strike,
                DecisionResult::Lost
            ),
            (
                DecisionSituation::Disadvantage,
                DecisionOption::Throw,
                DecisionResult::Survived
            ),
            (
                DecisionSituation::Disadvantage,
                DecisionOption::Throw,
                DecisionResult::Survived
            ),
        ]
    );
    assert_eq!(decisions[0].frames, 5, "不利フレーム数をそのまま持つ");
    assert_eq!(decisions[0].pressed, "LP", "押したボタンを落とさない");
}

/// 何もしなかった不利場面も機会として数える。分母から抜けると
/// 「暴れてばかり」に見えてしまう。
#[test]
fn a_minus_situation_without_an_action_is_still_an_opportunity() {
    let mut events = empty_events();
    events.presses_while_minus.push(minus_press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::Won,
    ));
    events.minus_situations.push(MinusSituationEvent {
        side: 1,
        frame: 200,
        minus_frames: 7,
        fastest_action: None,
        action_frame: None,
        pressed: String::new(),
        outcome: None,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: 200,
        round_no: 1,
    });

    let decisions = collect_decisions(&events, 1);

    assert_eq!(
        opportunities(&decisions, DecisionSituation::Disadvantage),
        2
    );
    let quiet = selections(
        &decisions,
        DecisionSituation::Disadvantage,
        DecisionOption::NoAttack,
    );
    assert_eq!(quiet.len(), 1);
    assert_eq!(quiet[0].frames, 7);
}

/// 最速の回答を取った場面は presses_while_minus が結末まで持つ。
/// minus_situations 側にも残っていると二重に数えてしまう。
#[test]
fn a_situation_with_a_recorded_action_is_not_counted_twice() {
    let mut events = empty_events();
    events.presses_while_minus.push(minus_press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::Won,
    ));
    events.minus_situations.push(MinusSituationEvent {
        side: 1,
        frame: 100,
        minus_frames: 5,
        fastest_action: Some(DefensiveActionKind::Strike),
        action_frame: Some(105),
        pressed: "LP".to_string(),
        outcome: Some(MinusPressOutcome::Won),
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: 100,
        round_no: 1,
    });

    let decisions = collect_decisions(&events, 1);

    assert_eq!(
        opportunities(&decisions, DecisionSituation::Disadvantage),
        1
    );
}

/// 有利フレームは、続く行動があったかで回答が決まる。ターンを渡したときだけ
/// 負けとする。
#[test]
fn an_advantage_is_read_from_whether_an_action_followed() {
    let mut events = empty_events();
    events
        .advantage_situations
        .push(advantage(100, Some(110), AdvantageOutcome::Continued));
    events.advantage_situations[0].pressed = "中P".to_string();
    events
        .advantage_situations
        .push(advantage(200, None, AdvantageOutcome::TurnLost));
    events
        .advantage_situations
        .push(advantage(300, None, AdvantageOutcome::Reset));

    let decisions = collect_decisions(&events, 1);

    let read: Vec<_> = decisions
        .iter()
        .map(|event| (event.option, event.result))
        .collect();
    assert_eq!(
        read,
        vec![
            (DecisionOption::Strike, DecisionResult::Survived),
            (DecisionOption::NoAttack, DecisionResult::Lost),
            (DecisionOption::NoAttack, DecisionResult::Survived),
        ]
    );
    assert_eq!(
        decisions[0].pressed, "中P",
        "有利を取った後に選んだ入力を落としている"
    );
}

/// 起き攻めは、攻めたかどうかだけを回答として写す。攻めなかったことによる
/// 損失は観測できないので、結末は常に「凌いだ」にする。
#[test]
fn okizeme_records_whether_pressure_followed_but_never_a_loss() {
    let mut events = empty_events();
    events
        .knockdowns
        .push(knockdown(100, OkizemeOutcome::Meaty));
    events
        .knockdowns
        .push(knockdown(200, OkizemeOutcome::Pressured));
    events
        .knockdowns
        .push(knockdown(300, OkizemeOutcome::Neutral));

    let decisions = collect_decisions(&events, 1);

    assert_eq!(opportunities(&decisions, DecisionSituation::Okizeme), 3);
    assert_eq!(
        decisions
            .iter()
            .map(|event| event.option)
            .collect::<Vec<_>>(),
        vec![
            DecisionOption::Strike,
            DecisionOption::Strike,
            DecisionOption::NoAttack
        ]
    );
    assert!(
        decisions
            .iter()
            .all(|event| event.result == DecisionResult::Survived),
        "攻めなかった損失は観測できない"
    );
}

/// 相手側の場面と、確度の足りない観測は取り込まない。どちらも入ると
/// 自分の傾向が歪む。
#[test]
fn the_other_side_and_unconfirmed_observations_are_left_out() {
    let mut events = empty_events();
    events.presses_while_minus.push(MinusPressEvent {
        side: 2,
        ..minus_press(100, DefensiveActionKind::Strike, MinusPressOutcome::Won)
    });
    events.presses_while_minus.push(MinusPressEvent {
        confidence: EventConfidence::Medium,
        ..minus_press(200, DefensiveActionKind::Strike, MinusPressOutcome::Won)
    });
    events.advantage_situations.push(AdvantageSituationEvent {
        side: 2,
        ..advantage(300, None, AdvantageOutcome::TurnLost)
    });
    events.knockdowns.push(KnockdownEvent {
        attacker: 2,
        ..knockdown(400, OkizemeOutcome::Meaty)
    });

    assert!(collect_decisions(&events, 1).is_empty());
}

/// 判断機会はフレーム順に並べる。時系列で読める形でないと、証拠として
/// 提示したときに追えない。
#[test]
fn decisions_come_back_in_frame_order() {
    let mut events = empty_events();
    events
        .knockdowns
        .push(knockdown(400, OkizemeOutcome::Meaty));
    events.presses_while_minus.push(minus_press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::Won,
    ));
    events
        .advantage_situations
        .push(advantage(250, Some(260), AdvantageOutcome::Continued));

    let frames: Vec<_> = collect_decisions(&events, 1)
        .iter()
        .map(|event| event.frame)
        .collect();

    assert_eq!(frames, vec![100, 250, 400]);
}

/// 偏りは「最も多かった回答」とその機会数で表す。同率のときは選択肢の
/// 定義順で先に来るものを返す（打撃 → 投げ → 攻撃なし）。
#[test]
fn the_bias_reports_the_most_chosen_option_with_its_denominator() {
    let mut events = empty_events();
    for frame in [100, 200, 300] {
        events.presses_while_minus.push(minus_press(
            frame,
            DefensiveActionKind::Throw,
            MinusPressOutcome::Won,
        ));
    }
    events.presses_while_minus.push(minus_press(
        400,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    ));

    let decisions = collect_decisions(&events, 1);

    assert_eq!(
        option_bias(&decisions, DecisionSituation::Disadvantage),
        Some((DecisionOption::Throw, 3, 4))
    );
    let strikes = selections(
        &decisions,
        DecisionSituation::Disadvantage,
        DecisionOption::Strike,
    );
    assert_eq!(losses(&strikes).len(), 1);
}

/// 機会が無い状況では偏りを語らない。0 件から割合を出すと、指摘が
/// 根拠を失う。
#[test]
fn a_situation_that_never_happened_has_no_bias() {
    let decisions = collect_decisions(&empty_events(), 1);

    assert_eq!(opportunities(&decisions, DecisionSituation::Advantage), 0);
    assert_eq!(option_bias(&decisions, DecisionSituation::Advantage), None);
}
