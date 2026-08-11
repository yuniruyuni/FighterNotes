//! 確定反撃をめぐる三つの指摘に対するテスト。
//!
//! ガードして反撃の猶予があったのに、取れなかった／出したが届かなかった／
//! 取れたが小さく終わった、の三つ。どれも「フレーム上は有利だった」だけ
//! では言えない。長い技の先端をガードした場合、時間の有利があっても届か
//! ないので、位置まで確認できた場面に限る。
//!
//! 三つとも失った HP ではなく機会の損失なので、重みの付け方が他の指摘と
//! 違う。ここを取り違えると、取り逃がした反撃が実際の被弾より重く並ぶ。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    AttackDamageConsistency, ContactEvent, DamageAttackEvidence, DamageEvent, MatchEvents,
    PunishChance, PunishOrigin, PunishOutcome, PunishReachability,
};
use crate::AdviceKind;

/// 反撃の機会。
fn chance(frame: u32, outcome: PunishOutcome, advantage: u32) -> PunishChance {
    PunishChance {
        frame,
        side: 1,
        advantage,
        outcome,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: frame.saturating_sub(20),
        recovery_end_frame: frame,
        source_contact_frame: Some(frame.saturating_sub(30)),
        attack_start_frame: Some(frame + 2),
        attack_active_frame: Some(frame + 8),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: "強P".to_string(),
        round_no: 1,
    }
}

fn events_with(chances: Vec<PunishChance>) -> MatchEvents {
    MatchEvents {
        punishes: chances,
        ..empty_events()
    }
}

// ── 取れなかった確定反撃 ─────────────────────────────────────────────────

/// 機会が無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_missed_chance() {
    assert!(detect_punish_missed(&empty_events(), 1, None).is_none());
}

/// 反撃を取れていれば見逃しではない。
#[test]
fn a_punish_that_landed_is_not_a_miss() {
    let events = events_with(vec![chance(100, PunishOutcome::Success, 10)]);

    assert!(detect_punish_missed(&events, 1, None).is_none());
}

/// 位置まで確認できていない機会は使わない。長い技の先端をガードした
/// 場合、時間の有利があっても届かない。
#[test]
fn a_chance_whose_range_was_not_confirmed_is_not_used() {
    let mut events = events_with(vec![chance(100, PunishOutcome::Missed, 10)]);
    events.punishes[0].reachability = PunishReachability::Unknown;

    assert!(detect_punish_missed(&events, 1, None).is_none());
}

/// 相手の機会は自分の話ではない。
#[test]
fn the_opponents_chances_are_not_yours() {
    let mut events = events_with(vec![chance(100, PunishOutcome::Missed, 10)]);
    events.punishes[0].side = 2;

    assert!(detect_punish_missed(&events, 1, None).is_none());
}

/// 一度でも見逃していれば指摘する。相手の技を覚えているかどうかの
/// 話なので、読み合いの結果とは違う。
#[test]
fn one_missed_punish_is_already_worth_saying() {
    let events = events_with(vec![chance(100, PunishOutcome::Missed, 10)]);

    let card = detect_punish_missed(&events, 1, None).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.evidence.len(), 1);
    assert!((card.severity - 0.04).abs() < 1e-6);
}

/// 失った HP ではなく取り逃がした利益なので、被ダメは空にする。
/// 埋めると、実際の被弾と同じ土俵で並ぶ。
#[test]
fn a_missed_punish_reports_no_health_lost() {
    let events = events_with(vec![chance(100, PunishOutcome::Missed, 10)]);

    let card = detect_punish_missed(&events, 1, None).expect("提示される");

    assert_eq!(card.hp_lost, None, "機会の損失を被ダメとして出している");
    assert!(card.severity > 0.0, "重みが付いていない");
}

/// 見逃した回数が多いほど重く扱う。
#[test]
fn missing_more_often_weighs_more() {
    let once = events_with(vec![chance(100, PunishOutcome::Missed, 10)]);
    let twice = events_with(vec![
        chance(100, PunishOutcome::Missed, 10),
        chance(600, PunishOutcome::Missed, 10),
    ]);

    let once = detect_punish_missed(&once, 1, None).expect("提示される");
    let twice = detect_punish_missed(&twice, 1, None).expect("提示される");

    assert!(twice.severity > once.severity, "回数が重みに効いていない");
    assert_ne!(once.title, twice.title, "見出しを書き分けていない");
}

/// キャラクターが分かれば、その有利フレームで間に合う技を挙げる。
#[test]
fn the_reachable_moves_are_suggested_when_the_character_is_known() {
    let events = events_with(vec![chance(100, PunishOutcome::Missed, 12)]);

    let known = detect_punish_missed(&events, 1, Some("LUKE")).expect("提示される");
    let unknown = detect_punish_missed(&events, 1, None).expect("提示される");

    assert!(
        known.description.contains("威力"),
        "技の候補を挙げていない: {}",
        known.description
    );
    assert_ne!(
        known.description, unknown.description,
        "キャラクターが分かっても同じ文面を出している"
    );
}

/// 知らないキャラクターでは候補を挙げない。適当な技名を出すより、
/// 一般的な指針を出す方がよい。
#[test]
fn an_unknown_character_gets_general_advice_instead() {
    let events = events_with(vec![chance(100, PunishOutcome::Missed, 12)]);

    let card = detect_punish_missed(&events, 1, Some("だれか")).expect("提示される");

    assert!(
        !card.description.contains("威力"),
        "知らないキャラクターの技を挙げている: {}",
        card.description
    );
}

/// 候補は最も厳しい機会に合わせる。一番有利が小さい場面で間に合う技で
/// なければ、全部の場面には使えない。
#[test]
fn the_suggestions_fit_the_tightest_chance() {
    let events = events_with(vec![
        chance(100, PunishOutcome::Missed, 20),
        chance(600, PunishOutcome::Missed, 5),
    ]);

    let card = detect_punish_missed(&events, 1, Some("LUKE")).expect("提示される");

    assert!(
        card.description.contains("有利 5F"),
        "最も厳しい機会に合わせていない: {}",
        card.description
    );
}

/// クリップには有利幅を出す。何フレーム余っていたのかが分からないと、
/// どの技なら間に合ったのか考えられない。
#[test]
fn the_missed_clip_says_how_much_time_there_was() {
    let events = events_with(vec![chance(100, PunishOutcome::Missed, 12)]);

    let card = detect_punish_missed(&events, 1, None).expect("提示される");

    assert!(
        card.evidence[0].label.contains("+12F"),
        "有利幅が出ていない: {}",
        card.evidence[0].label
    );
}

// ── 出したが届かなかった反撃 ─────────────────────────────────────────────

/// 空振りが無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_whiffed_punish() {
    assert!(detect_punish_fail(&empty_events(), 1, None).is_none());
}

/// 位置を確認できていない空振りは使わない。
#[test]
fn a_whiff_whose_range_was_not_confirmed_is_not_used() {
    let mut events = events_with(vec![chance(100, PunishOutcome::WhiffFail, 10)]);
    events.punishes[0].reachability = PunishReachability::Unknown;

    assert!(detect_punish_fail(&events, 1, None).is_none());
}

/// 一度きりは距離依存の技選択かもしれない。事実確認に留める。
#[test]
fn a_single_whiffed_punish_stays_an_observation() {
    let events = events_with(vec![chance(100, PunishOutcome::WhiffFail, 10)]);

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.severity - 0.03).abs() < 1e-6);
}

/// 同じ入力で繰り返し届いていなければ、技選択の話。
#[test]
fn the_same_input_whiffing_twice_is_a_diagnosis() {
    let events = events_with(vec![
        chance(100, PunishOutcome::WhiffFail, 10),
        chance(600, PunishOutcome::WhiffFail, 10),
    ]);

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Diagnosis,
        "同じ入力の繰り返しを見ていない"
    );
    assert!(
        card.description.contains("強P"),
        "繰り返した入力を出していない: {}",
        card.description
    );
}

/// 入力が毎回違えば、同じ癖とは言えない。回数だけを数えると、距離の
/// 違う二度の失敗が癖にされる。
#[test]
fn whiffing_with_different_inputs_is_not_one_habit() {
    let mut events = events_with(vec![
        chance(100, PunishOutcome::WhiffFail, 10),
        chance(600, PunishOutcome::WhiffFail, 10),
    ]);
    events.punishes[1].pressed = "中K".to_string();

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "別々の入力をまとめて癖にしている"
    );
}

/// 入力が記録されていない空振りは、繰り返しの数に入れない。
#[test]
fn a_whiff_without_a_recorded_input_is_not_counted_as_repetition() {
    let mut events = events_with(vec![
        chance(100, PunishOutcome::WhiffFail, 10),
        chance(600, PunishOutcome::WhiffFail, 10),
    ]);
    for punish in &mut events.punishes {
        punish.pressed = String::new();
    }

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Observation);
}

/// 空振りの後に被弾していれば、その分は実際の損失として数える。
#[test]
fn health_lost_after_the_whiff_is_counted() {
    let mut events = events_with(vec![chance(100, PunishOutcome::WhiffFail, 10)]);
    events.punishes[0].punished_drop = 0.18;

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.18));
    assert!((card.severity - 0.21).abs() < 1e-6);
    assert!(
        card.evidence[0].label.contains("18"),
        "被弾を出していない: {}",
        card.evidence[0].label
    );
    assert!(
        card.description.contains("合計 18%"),
        "説明の被ダメ率が違う: {}",
        card.description
    );
}

/// 被弾しなかった空振りには、被弾の話を付けない。
#[test]
fn a_whiff_that_cost_nothing_says_nothing_about_damage() {
    let events = events_with(vec![chance(100, PunishOutcome::WhiffFail, 10)]);

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert!(
        !card.evidence[0].label.contains("被弾"),
        "無い被弾を書いている: {}",
        card.evidence[0].label
    );
}

/// 取れている反撃の数も出す。全部外しているのか、たまに外すのかで
/// 話が変わる。
#[test]
fn the_number_of_punishes_that_landed_is_reported() {
    let events = events_with(vec![
        chance(100, PunishOutcome::WhiffFail, 10),
        chance(600, PunishOutcome::Success, 10),
        chance(1200, PunishOutcome::Success, 10),
    ]);

    let card = detect_punish_fail(&events, 1, None).expect("提示される");

    assert!(
        card.description.contains("確反成功は 2 回"),
        "取れている回数を出していない: {}",
        card.description
    );
}

/// 届かなかった反撃でも、候補技は最も小さい有利幅に合わせる。
#[test]
fn failed_punish_suggestions_use_the_tightest_advantage() {
    let events = events_with(vec![
        chance(100, PunishOutcome::WhiffFail, 20),
        chance(600, PunishOutcome::WhiffFail, 5),
    ]);

    let card = detect_punish_fail(&events, 1, Some("LUKE")).expect("提示される");

    assert!(
        card.description.contains("有利 5F"),
        "最も厳しい有利幅を使っていない: {}",
        card.description
    );
    assert!(
        card.description.contains("2LP"),
        "キャラクター別候補をカードへ渡していない: {}",
        card.description
    );
    assert!((card.severity - 0.06).abs() < 1e-6);
}

// ── 小さく終わった反撃 ───────────────────────────────────────────────────

fn dealt(frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 2,
        start_frame: frame,
        end_frame: frame + 40,
        pre_freeze_frame: frame,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn hit_contact(frame: u32) -> ContactEvent {
    ContactEvent {
        frame,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }
}

/// 反撃が取れて、与えたダメージが小さかった試合。
fn one_low_return(drop: f32) -> MatchEvents {
    let mut events = events_with(vec![chance(100, PunishOutcome::Success, 10)]);
    events.contacts = vec![hit_contact(105)];
    events.damage = vec![dealt(110, drop)];
    events
}

/// 接触の記録が無い動画では判断しない。何発当たったのかが分からない。
#[test]
fn without_contact_records_nothing_is_judged() {
    let mut events = one_low_return(0.06);
    events.contacts = vec![];

    assert!(detect_low_conversion(&events, 1).is_none());
}

/// 十分なリターンが取れていれば指摘しない。
#[test]
fn a_punish_that_paid_off_is_not_reported() {
    let at_the_edge = one_low_return(0.12);
    let below = one_low_return(0.11);

    assert!(
        detect_low_conversion(&at_the_edge, 1).is_none(),
        "閾値ちょうどを小さいリターンにしている"
    );
    assert!(
        detect_low_conversion(&below, 1).is_some(),
        "閾値を下回るリターンを見ていない"
    );
}

/// 与えたダメージが記録されていなければ、小さいとも言えない。
#[test]
fn a_punish_with_no_recorded_damage_is_not_judged() {
    let mut events = one_low_return(0.06);
    events.damage = vec![];

    assert!(detect_low_conversion(&events, 1).is_none());
}

/// 自分が受けた被弾は、自分が与えたリターンではない。
#[test]
fn damage_you_took_is_not_your_return() {
    let mut events = one_low_return(0.06);
    events.damage[0].victim = 1;

    assert!(detect_low_conversion(&events, 1).is_none());
}

/// 反撃から離れた被弾は、その反撃の結果ではない。
#[test]
fn damage_far_from_the_punish_is_not_its_return() {
    let mut inside = one_low_return(0.06);
    inside.damage = vec![dealt(220, 0.06)];
    assert!(
        detect_low_conversion(&inside, 1).is_some(),
        "窓の内側の結果を落としている"
    );

    let mut outside = one_low_return(0.06);
    outside.damage = vec![dealt(221, 0.06)];
    assert!(
        detect_low_conversion(&outside, 1).is_none(),
        "窓の外の被弾を結果にしている"
    );
}

/// 反撃が一度きりなら、ゲージ温存や位置取りの判断かもしれない。
#[test]
fn a_single_low_return_stays_an_observation() {
    let card = detect_low_conversion(&one_low_return(0.06), 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.severity - 0.03).abs() < 1e-6);
}

/// 同じ入力で繰り返し小さく終わっていれば、コンボに繋ぐ話。
#[test]
fn the_same_input_ending_small_twice_is_a_diagnosis() {
    let mut events = events_with(vec![
        chance(100, PunishOutcome::Success, 10),
        chance(600, PunishOutcome::Success, 10),
    ]);
    events.contacts = vec![hit_contact(105), hit_contact(605)];
    events.damage = vec![dealt(110, 0.06), dealt(610, 0.05)];

    let card = detect_low_conversion(&events, 1).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert!((card.severity - 0.06).abs() < 1e-6);
    assert!(
        card.description.contains("強P"),
        "繰り返した入力を出していない: {}",
        card.description
    );
    assert!(
        card.description.contains("合計 11%"),
        "小リターンの合計が違う: {}",
        card.description
    );
}

/// 与えられたはずの分は機会の損失なので、被ダメは空にする。
#[test]
fn a_low_return_reports_no_health_lost() {
    let card = detect_low_conversion(&one_low_return(0.06), 1).expect("提示される");

    assert_eq!(card.hp_lost, None, "機会の損失を被ダメとして出している");
    assert!(card.severity > 0.0);
}

/// 反撃の結果は表示の読みまで含めて断定しきれないので、確度は控えめ。
#[test]
fn the_low_return_card_keeps_its_confidence_modest() {
    let card = detect_low_conversion(&one_low_return(0.06), 1).expect("提示される");

    assert_eq!(
        card.confidence,
        crate::match_events::EventConfidence::Medium
    );
}

/// ゲーム内表示まで読めていれば、その数字を添える。
#[test]
fn a_reliable_in_game_reading_is_added() {
    let mut events = one_low_return(0.06);
    events.attack_evidence.damage = vec![DamageAttackEvidence {
        victim: 2,
        attacker: 1,
        damage_start_frame: 110,
        sequence_start_frame: 110,
        sequence_end_frame: 150,
        combo_damage: 900,
        sequence_count: 1,
        final_scaling_percent: 100,
        starter_attribute: None,
        final_attribute: crate::attack_info::AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: crate::match_events::EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    }];

    let card = detect_low_conversion(&events, 1).expect("提示される");

    assert!(
        card.description.contains("900"),
        "読めた数字を出していない: {}",
        card.description
    );
}

/// 読めていなければ、その話は書かない。
#[test]
fn nothing_is_said_about_a_reading_that_was_not_taken() {
    let card = detect_low_conversion(&one_low_return(0.06), 1).expect("提示される");

    assert!(
        !card.description.contains("ゲーム内表示"),
        "読めていない話をしている: {}",
        card.description
    );
}

/// 取れている反撃の総数を分母に出す。全部が小さいのか、たまに小さいのかで
/// 話が変わる。
#[test]
fn the_total_number_of_punishes_is_the_denominator() {
    let mut events = events_with(vec![
        chance(100, PunishOutcome::Success, 10),
        chance(600, PunishOutcome::Success, 10),
    ]);
    events.contacts = vec![hit_contact(105), hit_contact(605)];
    events.damage = vec![dealt(110, 0.06), dealt(610, 0.30)];

    let card = detect_low_conversion(&events, 1).expect("提示される");

    assert!(
        card.description.contains("確反成功 2 回中"),
        "分母を出していない: {}",
        card.description
    );
    assert_eq!(card.evidence.len(), 1, "十分なリターンまで並べている");
}
