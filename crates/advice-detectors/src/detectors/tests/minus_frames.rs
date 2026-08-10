//! 不利フレーム後の回答に対するテスト。
//!
//! ガードして不利を背負った状態からの回答は、打撃・投げ・ガード継続など
//! いくつもある。どれか一つに偏っていると相手に読まれるが、偏りは
//! 「その回答を選んだ回数」だけでは分からない。分母、つまり同じ状況が
//! 何回あったかと並べて初めて言える。
//!
//! 分母を取り違えると、一度の被弾が「偏っている」になり、あるいは
//! 明らかな癖が見逃される。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    DefensiveActionKind, EventConfidence, MatchEvents, MinusPressEvent, MinusPressOutcome,
    MinusSituationEvent,
};
use crate::AdviceKind;

/// 不利から押した場面。
fn press(
    frame: u32,
    kind: DefensiveActionKind,
    outcome: MinusPressOutcome,
    drop: f32,
) -> MinusPressEvent {
    MinusPressEvent {
        side: 1,
        frame,
        minus_frames: 5,
        pressed: "弱".to_string(),
        action_kind: kind,
        outcome,
        drop,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(10),
        round_no: 1,
    }
}

/// 不利を背負ったが、打撃も投げも選ばなかった場面。分母にだけ効く。
fn other_answer(frame: u32) -> MinusSituationEvent {
    MinusSituationEvent {
        side: 1,
        frame,
        minus_frames: 5,
        fastest_action: None,
        action_frame: None,
        pressed: String::new(),
        outcome: None,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(10),
        round_no: 1,
    }
}

/// 打撃を選んだ場面を分母にも残す。
fn chose_strike(frame: u32) -> MinusSituationEvent {
    MinusSituationEvent {
        fastest_action: Some(DefensiveActionKind::Strike),
        ..other_answer(frame)
    }
}

fn events_with(presses: Vec<MinusPressEvent>, situations: Vec<MinusSituationEvent>) -> MatchEvents {
    MatchEvents {
        presses_while_minus: presses,
        minus_situations: situations,
        ..empty_events()
    }
}

// ── 打撃を選んだ場面 ─────────────────────────────────────────────────────

/// 押して勝った、あるいは逃げ切った場面だけでは指摘しない。不利から
/// 押すこと自体は正当な選択肢。
#[test]
fn a_press_that_costs_nothing_is_not_reported() {
    let events = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Strike,
                MinusPressOutcome::Won,
                0.0,
            ),
            press(
                300,
                DefensiveActionKind::Strike,
                MinusPressOutcome::GotAway,
                0.0,
            ),
        ],
        vec![],
    );

    assert!(detect_press_while_minus(&events, 1).is_none());
}

/// 狩られた場面が一度あれば事実として出すが、偏りとまでは言わない。
#[test]
fn a_single_counter_hit_stays_an_observation() {
    let events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );

    let card = detect_press_while_minus(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で偏りと呼んでいる");
    assert_eq!(card.hp_lost, Some(0.12));
}

/// 機会の大半で同じ回答を選び、何度も狩られていれば偏り。
#[test]
fn choosing_the_same_answer_almost_every_time_is_a_bias() {
    let events = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.12,
            ),
            press(
                300,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.10,
            ),
            press(
                500,
                DefensiveActionKind::Strike,
                MinusPressOutcome::Won,
                0.0,
            ),
            press(
                700,
                DefensiveActionKind::Strike,
                MinusPressOutcome::Won,
                0.0,
            ),
        ],
        vec![],
    );

    let card = detect_press_while_minus(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis, "偏りと呼べていない");
    assert!((card.hp_lost.expect("損失がある") - 0.22).abs() < 1e-6);
}

/// 同じ回数押していても、機会の方が多ければ偏りではない。分母を
/// 見落とすと、たまたま狩られた数回が癖にされる。
#[test]
fn the_same_presses_are_not_a_bias_when_there_were_more_chances() {
    let presses = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        ),
        press(
            300,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.10,
        ),
        press(
            500,
            DefensiveActionKind::Strike,
            MinusPressOutcome::Won,
            0.0,
        ),
    ];
    // 打撃を選んだ 3 回に加え、選ばなかった機会が 7 回。選択率 30%。
    let mut situations: Vec<_> = [100, 300, 500].iter().map(|f| chose_strike(*f)).collect();
    situations.extend((0..7).map(|index| other_answer(1000 + index * 100)));

    let card = detect_press_while_minus(&events_with(presses, situations), 1).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "選択率を見ずに偏りと呼んでいる"
    );
}

/// 機会が数えられていれば、選択率をそのまま説明に出す。
#[test]
fn the_description_reports_how_often_the_answer_was_chosen() {
    let presses = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        ),
        press(
            300,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.10,
        ),
    ];
    let mut situations: Vec<_> = [100, 300].iter().map(|f| chose_strike(*f)).collect();
    situations.extend((0..2).map(|index| other_answer(1000 + index * 100)));

    let card = detect_press_while_minus(&events_with(presses, situations), 1).expect("提示される");

    assert!(
        card.description.contains("50%"),
        "選択率が出ていない: {}",
        card.description
    );
}

/// 最も多く押していたボタンを出す。何を押す癖なのかが分からないと、
/// 直しようがない。
#[test]
fn the_most_pressed_button_is_named() {
    let mut presses = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        ),
        press(
            300,
            DefensiveActionKind::Strike,
            MinusPressOutcome::Won,
            0.0,
        ),
        press(
            500,
            DefensiveActionKind::Strike,
            MinusPressOutcome::Won,
            0.0,
        ),
    ];
    presses[0].pressed = "中".to_string();
    presses[1].pressed = "強".to_string();
    presses[2].pressed = "強".to_string();

    let card = detect_press_while_minus(&events_with(presses, vec![]), 1).expect("提示される");

    assert!(
        card.description.contains('強'),
        "多かったボタンを出していない: {}",
        card.description
    );
}

/// 相手の押しは自分の話ではない。
#[test]
fn the_opponents_presses_are_not_reported() {
    let mut events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );
    events.presses_while_minus[0].side = 2;

    assert!(detect_press_while_minus(&events, 1).is_none());
}

/// 入力まで確認できていない場面は数えない。押したかどうかが曖昧な
/// まま「偏っている」とは言えない。
#[test]
fn an_unconfirmed_press_is_not_counted() {
    let mut events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );
    events.presses_while_minus[0].confidence = EventConfidence::Low;

    assert!(detect_press_while_minus(&events, 1).is_none());
}

/// 逃げ切った回数も重みに入れる。狩られていなくても、同じ回答を
/// 押し続けていること自体が読まれる材料になる。
#[test]
fn getting_away_still_adds_a_little_weight() {
    let bare = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );
    let with_escapes = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.12,
            ),
            press(
                300,
                DefensiveActionKind::Strike,
                MinusPressOutcome::GotAway,
                0.0,
            ),
        ],
        vec![],
    );

    let bare = detect_press_while_minus(&bare, 1).expect("提示される");
    let with_escapes = detect_press_while_minus(&with_escapes, 1).expect("提示される");

    assert_eq!(bare.hp_lost, with_escapes.hp_lost, "損失は同じはず");
    assert!(
        with_escapes.severity > bare.severity,
        "逃げ切った回数が重みに効いていない"
    );
}

/// 偏りかどうかで文面を書き分ける。同じ文だと、一度の被弾と癖が
/// 同じ強さで読まれる。
#[test]
fn the_wording_changes_when_it_becomes_a_bias() {
    let once = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );
    let biased = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.12,
            ),
            press(
                300,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.10,
            ),
            press(
                500,
                DefensiveActionKind::Strike,
                MinusPressOutcome::Won,
                0.0,
            ),
            press(
                700,
                DefensiveActionKind::Strike,
                MinusPressOutcome::Won,
                0.0,
            ),
        ],
        vec![],
    );

    let once = detect_press_while_minus(&once, 1).expect("提示される");
    let biased = detect_press_while_minus(&biased, 1).expect("提示される");

    assert_eq!(once.id, biased.id);
    assert_ne!(once.title, biased.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, biased.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, biased.practice, "練習方法を書き分けていない");
}

/// クリップの見出しには、不利幅・押したボタン・失った HP を出す。
/// どれが欠けても、映像を見る前に何が起きたか分からない。
#[test]
fn the_clip_label_says_what_happened() {
    let events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );

    let card = detect_press_while_minus(&events, 1).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains('5'), "不利幅が出ていない: {label}");
    assert!(label.contains('弱'), "押したボタンが出ていない: {label}");
    assert!(label.contains("12"), "失った HP が出ていない: {label}");
}

// ── 投げを選んだ場面 ─────────────────────────────────────────────────────

/// 投げの指摘は打撃とは別の話。打撃で狩られた場面が投げの指摘に
/// 出てはいけない。
#[test]
fn the_strike_and_throw_cards_do_not_share_situations() {
    let events = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Strike,
                MinusPressOutcome::CounterHit,
                0.12,
            ),
            press(
                300,
                DefensiveActionKind::Throw,
                MinusPressOutcome::CounterHit,
                0.20,
            ),
        ],
        vec![],
    );

    let strike = detect_press_while_minus(&events, 1).expect("打撃の指摘");
    let throws = detect_throw_while_minus(&events, 1).expect("投げの指摘");

    assert_eq!(strike.hp_lost, Some(0.12));
    assert_eq!(throws.hp_lost, Some(0.20));
    assert_ne!(strike.id, throws.id);
}

/// 投げ側も、一度きりなら事実確認に留める。
#[test]
fn a_single_countered_throw_stays_an_observation() {
    let events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Throw,
            MinusPressOutcome::CounterHit,
            0.20,
        )],
        vec![],
    );

    let card = detect_throw_while_minus(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.hp_lost, Some(0.20));
}

/// 投げ側も、機会の大半で選んで何度も狩られていれば偏り。
#[test]
fn choosing_the_throw_almost_every_time_is_a_bias() {
    let events = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Throw,
                MinusPressOutcome::CounterHit,
                0.20,
            ),
            press(
                300,
                DefensiveActionKind::Throw,
                MinusPressOutcome::CounterHit,
                0.15,
            ),
            press(500, DefensiveActionKind::Throw, MinusPressOutcome::Won, 0.0),
            press(700, DefensiveActionKind::Throw, MinusPressOutcome::Won, 0.0),
        ],
        vec![],
    );

    let card = detect_throw_while_minus(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
}

/// 投げ側も、一度きりと偏りで文面を書き分ける。
#[test]
fn the_throw_wording_changes_when_it_becomes_a_bias() {
    let once = events_with(
        vec![press(
            100,
            DefensiveActionKind::Throw,
            MinusPressOutcome::CounterHit,
            0.20,
        )],
        vec![],
    );
    let biased = events_with(
        vec![
            press(
                100,
                DefensiveActionKind::Throw,
                MinusPressOutcome::CounterHit,
                0.20,
            ),
            press(
                300,
                DefensiveActionKind::Throw,
                MinusPressOutcome::CounterHit,
                0.15,
            ),
            press(500, DefensiveActionKind::Throw, MinusPressOutcome::Won, 0.0),
            press(700, DefensiveActionKind::Throw, MinusPressOutcome::Won, 0.0),
        ],
        vec![],
    );

    let once = detect_throw_while_minus(&once, 1).expect("提示される");
    let biased = detect_throw_while_minus(&biased, 1).expect("提示される");

    assert_eq!(once.id, biased.id);
    assert_ne!(once.title, biased.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, biased.description,
        "説明を書き分けていない"
    );
}

// ── 機会の数え方 ─────────────────────────────────────────────────────────

/// 機会の記録が無い動画では、押した回数そのものを分母にする。分母が
/// 0 になると割合が計算できない。
#[test]
fn without_recorded_chances_the_presses_are_the_denominator() {
    let events = events_with(
        vec![press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        )],
        vec![],
    );

    let card = detect_press_while_minus(&events, 1).expect("提示される");

    assert!(
        card.description.contains("100%"),
        "分母を押した回数にしていない: {}",
        card.description
    );
}

/// 記録された機会が選んだ回数より少ない場合でも、選択率が 100% を
/// 超えてはいけない。
#[test]
fn the_share_never_exceeds_all_the_chances() {
    let presses = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        ),
        press(
            300,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.10,
        ),
    ];
    // 機会は 1 件しか記録されていない。
    let card = detect_press_while_minus(&events_with(presses, vec![chose_strike(100)]), 1)
        .expect("提示される");

    assert!(
        !card.description.contains("200%"),
        "選択率が 100% を超えている: {}",
        card.description
    );
}

/// 相手の機会は自分の分母に入れない。入れると選択率が薄まって、
/// 偏りが見逃される。
#[test]
fn the_opponents_chances_do_not_dilute_the_share() {
    let presses = vec![
        press(
            100,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.12,
        ),
        press(
            300,
            DefensiveActionKind::Strike,
            MinusPressOutcome::CounterHit,
            0.10,
        ),
        press(
            500,
            DefensiveActionKind::Strike,
            MinusPressOutcome::Won,
            0.0,
        ),
        press(
            700,
            DefensiveActionKind::Strike,
            MinusPressOutcome::Won,
            0.0,
        ),
    ];
    let mut situations: Vec<_> = [100, 300, 500, 700]
        .iter()
        .map(|f| chose_strike(*f))
        .collect();
    situations.extend((0..7).map(|index| {
        let mut theirs = other_answer(1000 + index * 100);
        theirs.side = 2;
        theirs
    }));

    let card = detect_press_while_minus(&events_with(presses, situations), 1).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Diagnosis,
        "相手の機会で分母が薄まっている"
    );
}
