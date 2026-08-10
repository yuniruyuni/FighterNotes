//! 投げが通らなかった場面の検出に対するテスト。
//!
//! 投げが失敗する理由は二つあり、指摘としての意味が違う。間合いの外で
//! 空振ったのなら距離の取り方の話、実行できたのに無敵技で潰されたのなら
//! 読み合いの話。混ぜると、読み負けを「投げの押しすぎ」と呼ぶことになる。
//!
//! どちらも一度きりでは事実確認に留め、繰り返して初めて診断にする。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    DamageEvent, EventConfidence, MatchEvents, ThrowActionEvent, ThrowOutcome,
};
use crate::AdviceKind;

/// 自分の投げ。実行まで確認できた前提で組む。
fn throw(frame: u32, outcome: ThrowOutcome) -> ThrowActionEvent {
    ThrowActionEvent {
        thrower: 1,
        input_frame: frame,
        startup_frame: Some(frame + 2),
        active_frame: Some(frame + 5),
        outcome,
        damage: 0.0,
        approach: Default::default(),
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 自分が受けた被弾。
fn damage(frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame: frame,
        end_frame: frame + 30,
        pre_freeze_frame: frame,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn events_with(throws: Vec<ThrowActionEvent>, damages: Vec<DamageEvent>) -> MatchEvents {
    MatchEvents {
        throw_actions: throws,
        damage: damages,
        ..empty_events()
    }
}

// ── 投げ空振りからの被弾 ─────────────────────────────────────────────────

/// 空振っただけで何も起きていない場面は指摘しない。間合いを測る手段
/// としての空振りは正当。
#[test]
fn a_whiff_that_costs_nothing_is_not_reported() {
    let events = events_with(vec![throw(100, ThrowOutcome::ExecutedWhiff)], vec![]);

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// 一度きりの被弾は読み負けと区別できないので、事実確認に留める。
#[test]
fn a_single_punished_whiff_stays_an_observation() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );

    let card = detect_throw_whiff_punished(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で診断にしている");
    assert_eq!(card.hp_lost, Some(0.15));
    assert_eq!(card.evidence.len(), 1);
}

/// 繰り返していれば、投げを押す距離とタイミングの話として診断にする。
#[test]
fn repeated_punished_whiffs_become_a_diagnosis() {
    let events = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(600, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(120, 0.15), damage(620, 0.20)],
    );

    let card = detect_throw_whiff_punished(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(
        card.kind,
        AdviceKind::Diagnosis,
        "繰り返しを診断にしていない"
    );
    assert_eq!(card.evidence.len(), 2);
    assert!((card.hp_lost.expect("損失がある") - 0.35).abs() < 1e-6);
}

/// 続けて空振ってから被弾した場面は、一連の出来事として一つにまとめる。
/// 別々に数えると、同じ被弾を二重に計上する。
#[test]
fn consecutive_whiffs_into_one_hit_are_one_exchange() {
    let events = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(140, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(180, 0.15)],
    );

    let card = detect_throw_whiff_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence.len(), 1, "同じ被弾を二件に割っている");
    assert_eq!(card.hp_lost, Some(0.15), "同じ被弾を二重に数えている");
    assert_eq!(
        card.kind,
        AdviceKind::Diagnosis,
        "空振りの回数は数えられていない"
    );
}

/// まとめた場面の見出しには、空振った回数を出す。
#[test]
fn a_grouped_exchange_says_how_many_whiffs_it_holds() {
    let events = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(140, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(180, 0.15)],
    );

    let card = detect_throw_whiff_punished(&events, 1).expect("提示される");

    assert!(
        card.evidence[0].label.contains("2回"),
        "回数が出ていない: {}",
        card.evidence[0].label
    );
}

/// 空振りより前の被弾は関係が無い。時間の向きを取り違えると、
/// 被弾のあとの空振りを原因として並べることになる。
#[test]
fn damage_before_the_whiff_is_not_its_consequence() {
    let events = events_with(
        vec![throw(200, ThrowOutcome::ExecutedWhiff)],
        vec![damage(100, 0.15)],
    );

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// 空振りから離れた被弾も関係が無い。窓を広げすぎると、無関係な
/// 被弾がすべて投げのせいになる。
#[test]
fn damage_far_after_the_whiff_is_a_separate_situation() {
    let inside = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(195, 0.15)],
    );
    let outside = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(196, 0.15)],
    );

    assert!(
        detect_throw_whiff_punished(&inside, 1).is_some(),
        "窓の中の被弾を落としている"
    );
    assert!(
        detect_throw_whiff_punished(&outside, 1).is_none(),
        "窓の外の被弾を拾っている"
    );
}

/// 相手の投げは自分の話ではない。
#[test]
fn the_opponents_whiffs_are_not_reported() {
    let mut events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    events.throw_actions[0].thrower = 2;

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// 相手が受けた被弾も自分の話ではない。
#[test]
fn damage_taken_by_the_opponent_is_not_counted() {
    let mut events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    events.damage[0].victim = 2;

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// ラウンドをまたいだ被弾は繋がらない。フレーム番号が近くても
/// 別のラウンドなら別の場面。
#[test]
fn damage_in_another_round_does_not_belong_to_the_whiff() {
    let mut events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    events.damage[0].round_no = 2;

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// 実行まで確認できていない投げは扱わない。入力だけが見えた場面から
/// 「空振った」とは言えない。
#[test]
fn a_throw_that_was_not_confirmed_is_not_reported() {
    let mut events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    events.throw_actions[0].confidence = EventConfidence::Low;

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

/// 成立した投げは空振りではない。
#[test]
fn a_throw_that_landed_is_not_a_whiff() {
    let mut events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    events.throw_actions[0].outcome = ThrowOutcome::Hit;

    assert!(detect_throw_whiff_punished(&events, 1).is_none());
}

// ── 投げが無敵技に潰された場面 ───────────────────────────────────────────

/// 実行できた投げが無敵技で潰されたのは、間合いの話ではなく読み合いの話。
/// 一度きりなら事実確認に留める。
#[test]
fn a_single_invincible_counter_stays_an_observation() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::InterruptedByInvincible)],
        vec![damage(110, 0.25)],
    );

    let card = detect_throw_interrupted_by_invincible(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.hp_lost, Some(0.25));
    assert!(
        card.description.contains("空振りではありません"),
        "間合いの話と混ぜている"
    );
}

/// 繰り返していても断定はしない。同じ起き攻めで投げに偏っていたのか、
/// 別々の読み合いでかみ合っただけかは、この情報からは分からない。
#[test]
fn repeated_invincible_counters_stay_an_observation() {
    let events = events_with(
        vec![
            throw(100, ThrowOutcome::InterruptedByInvincible),
            throw(600, ThrowOutcome::InterruptedByInvincible),
        ],
        vec![damage(110, 0.25), damage(610, 0.20)],
    );

    let card = detect_throw_interrupted_by_invincible(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "断定に踏み込んでいる");
    assert_eq!(card.evidence.len(), 2);
    assert!((card.hp_lost.expect("損失がある") - 0.45).abs() < 1e-6);
}

/// 無敵技は投げの発生とほぼ同時に始まるので、直前の被弾も同じ場面。
#[test]
fn damage_a_moment_before_the_throw_is_still_the_same_exchange() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::InterruptedByInvincible)],
        vec![damage(103, 0.25)],
    );

    assert!(
        detect_throw_interrupted_by_invincible(&events, 1).is_some(),
        "同時に起きた被弾を落としている"
    );
}

/// それより前の被弾は別の場面。
#[test]
fn damage_well_before_the_throw_is_a_separate_situation() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::InterruptedByInvincible)],
        vec![damage(102, 0.25)],
    );

    assert!(detect_throw_interrupted_by_invincible(&events, 1).is_none());
}

/// 潰された投げが無い場面では何も出さない。
#[test]
fn nothing_is_reported_without_an_invincible_counter() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );

    assert!(detect_throw_interrupted_by_invincible(&events, 1).is_none());
}

/// 二つの指摘は同じ場面を取り合わない。空振りと無敵技負けは別の話なので、
/// 片方の場面がもう片方にも出ると、同じ被弾を二回指摘することになる。
#[test]
fn the_two_cards_do_not_claim_the_same_exchange() {
    let events = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(600, ThrowOutcome::InterruptedByInvincible),
        ],
        vec![damage(120, 0.15), damage(610, 0.25)],
    );

    let whiff = detect_throw_whiff_punished(&events, 1).expect("空振りの指摘");
    let invincible = detect_throw_interrupted_by_invincible(&events, 1).expect("無敵技の指摘");

    assert_eq!(whiff.hp_lost, Some(0.15));
    assert_eq!(invincible.hp_lost, Some(0.25));
}

/// 失った HP の大きい場面ほど重く扱う。並べ替えの基準になる。
#[test]
fn a_costlier_exchange_weighs_more() {
    let light = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.10)],
    );
    let heavy = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.30)],
    );

    let light = detect_throw_whiff_punished(&light, 1).expect("提示される");
    let heavy = detect_throw_whiff_punished(&heavy, 1).expect("提示される");

    assert!(
        heavy.severity > light.severity,
        "損失の大きさが重みに効いていない"
    );
}

/// クリップは空振りの入力から被弾の終わりまで。手前を切ると、なぜ
/// 被弾したのかが映らない。
#[test]
fn the_clip_runs_from_the_input_to_the_end_of_the_hit() {
    let events = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );

    let card = detect_throw_whiff_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence[0].frame, 100, "入力から始まっていない");
    assert_eq!(
        card.evidence[0].end_frame,
        Some(150),
        "被弾の終わりまで映していない"
    );
}

/// 一度きりと繰り返しでは、見出しも説明も練習方法も書き分ける。同じ文面を
/// 出すと、単発の読み負けが癖として読まれる。
#[test]
fn the_wording_changes_between_one_time_and_a_habit() {
    let once = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.15)],
    );
    let habit = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(600, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(120, 0.15), damage(620, 0.15)],
    );

    let once = detect_throw_whiff_punished(&once, 1).expect("提示される");
    let habit = detect_throw_whiff_punished(&habit, 1).expect("提示される");

    assert_eq!(once.id, habit.id, "同じ指摘に別の id を振っている");
    assert_ne!(once.title, habit.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, habit.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, habit.practice, "練習方法を書き分けていない");
}

/// 無敵技に負けた場面も、一度きりと繰り返しで書き分ける。
#[test]
fn the_invincible_wording_changes_with_repetition() {
    let once = events_with(
        vec![throw(100, ThrowOutcome::InterruptedByInvincible)],
        vec![damage(110, 0.25)],
    );
    let habit = events_with(
        vec![
            throw(100, ThrowOutcome::InterruptedByInvincible),
            throw(600, ThrowOutcome::InterruptedByInvincible),
        ],
        vec![damage(110, 0.25), damage(610, 0.25)],
    );

    let once = detect_throw_interrupted_by_invincible(&once, 1).expect("提示される");
    let habit = detect_throw_interrupted_by_invincible(&habit, 1).expect("提示される");

    assert_eq!(once.id, habit.id);
    assert_ne!(once.title, habit.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, habit.description,
        "説明を書き分けていない"
    );
}

/// 同じ損失でも、空振りを重ねた分だけ重く扱う。回数を重みに入れないと、
/// 一回で被弾した場面と何度も押して被弾した場面が並んでしまう。
#[test]
fn repeating_the_whiff_adds_to_the_weight() {
    let once = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(180, 0.15)],
    );
    let twice = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(140, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(180, 0.15)],
    );

    let once = detect_throw_whiff_punished(&once, 1).expect("提示される");
    let twice = detect_throw_whiff_punished(&twice, 1).expect("提示される");

    assert_eq!(once.hp_lost, twice.hp_lost, "損失は同じはず");
    assert!(
        twice.severity > once.severity,
        "空振りの回数が重みに効いていない: {} / {}",
        twice.severity,
        once.severity
    );
    assert!(
        (twice.severity - once.severity - 0.02).abs() < 1e-6,
        "一回あたりの重みが変わっている"
    );
}

/// 重みの主役は失った HP。回数の分だけで大きな損失を追い越しては
/// 並び順が壊れる。
#[test]
fn the_lost_health_still_dominates_the_weight() {
    let many_whiffs = events_with(
        vec![
            throw(100, ThrowOutcome::ExecutedWhiff),
            throw(140, ThrowOutcome::ExecutedWhiff),
            throw(170, ThrowOutcome::ExecutedWhiff),
        ],
        vec![damage(190, 0.05)],
    );
    let one_big_hit = events_with(
        vec![throw(100, ThrowOutcome::ExecutedWhiff)],
        vec![damage(120, 0.30)],
    );

    let many = detect_throw_whiff_punished(&many_whiffs, 1).expect("提示される");
    let big = detect_throw_whiff_punished(&one_big_hit, 1).expect("提示される");

    assert!(
        big.severity > many.severity,
        "回数が損失を追い越している: {} / {}",
        many.severity,
        big.severity
    );
}
