//! ガード入力が外れて被弾した場面に対するテスト。
//!
//! ガード方向を握っていたのに途中で外れた被弾は、入力の癖かもしれないし、
//! 中下段や投げを読んで意図的に動いた結果かもしれない。同じ方向へ
//! 繰り返し外れて初めて癖として扱う。
//!
//! 投げは「ガードが外れた」話ではない。ガードしていても投げられるので、
//! 混ぜると入力の癖と読み合いの結果が区別できなくなる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::attack_info::AttackAttribute;
use crate::match_events::{
    AttackDamageConsistency, DamageAttackEvidence, DamageEvent, EventConfidence, GuardBreakEvent,
    MatchEvents,
};
use crate::AdviceKind;

/// ガード入力が外れた被弾。
fn guard_break(frame: u32, from: &str, to: &str, drop: f32) -> GuardBreakEvent {
    GuardBreakEvent {
        side: 1,
        frame,
        drop,
        guard_dir: from.to_string(),
        broke_to: to.to_string(),
        round_no: 1,
    }
}

/// その被弾に対応する HP イベント。攻撃属性を引くために要る。
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

/// ゲーム内表示から読めた攻撃の素性。
fn attack(frame: u32, attribute: AttackAttribute) -> DamageAttackEvidence {
    DamageAttackEvidence {
        victim: 1,
        attacker: 2,
        damage_start_frame: frame,
        sequence_start_frame: frame,
        sequence_end_frame: frame + 30,
        combo_damage: 1800,
        sequence_count: 1,
        final_scaling_percent: 100,
        starter_attribute: Some(attribute),
        final_attribute: attribute,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    }
}

fn events_with(breaks: Vec<GuardBreakEvent>, attacks: Vec<DamageAttackEvidence>) -> MatchEvents {
    let mut events = empty_events();
    events.damage = breaks
        .iter()
        .map(|event| damage(event.frame, event.drop))
        .collect();
    events.guard_breaks = breaks;
    events.attack_evidence.damage = attacks;
    events
}

/// 外れた場面が無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_broken_guard() {
    assert!(detect_guard_break(&empty_events(), 1).is_none());
}

/// 一度きりは、読み合いで意図的に動いた可能性と区別が付かない。
#[test]
fn a_single_break_stays_an_observation() {
    let events = events_with(vec![guard_break(100, "DR", "UR", 0.15)], vec![]);

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で癖と呼んでいる");
    assert_eq!(card.hp_lost, Some(0.15));
}

/// 同じ方向へ繰り返し外れていれば入力の癖。
#[test]
fn the_same_transition_twice_is_a_habit() {
    let events = events_with(
        vec![
            guard_break(100, "DR", "UR", 0.15),
            guard_break(1000, "DR", "UR", 0.12),
        ],
        vec![],
    );

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis, "繰り返しを拾えていない");
    assert!((card.hp_lost.expect("損失がある") - 0.27).abs() < 1e-6);
}

/// 外れた方向が毎回違えば癖ではない。回数だけを数えると、たまたま
/// 二度動いただけで入力の癖にされる。
#[test]
fn breaking_in_different_directions_is_not_one_habit() {
    let events = events_with(
        vec![
            guard_break(100, "DR", "UR", 0.15),
            guard_break(1000, "R", "N", 0.12),
        ],
        vec![],
    );

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "別々の遷移をまとめて癖にしている"
    );
    assert_eq!(card.hp_lost, Some(0.27), "損失は全件を合計する");
}

/// 最も多い遷移を癖として名指しする。どの方向へ外れているかが
/// 分からないと直しようがない。
#[test]
fn the_most_common_transition_is_named() {
    let events = events_with(
        vec![
            guard_break(100, "DR", "UR", 0.15),
            guard_break(1000, "DR", "UR", 0.12),
            guard_break(2000, "R", "N", 0.10),
        ],
        vec![],
    );

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(
        card.description.contains("↘"),
        "遷移の元を出していない: {}",
        card.description
    );
    assert!(
        card.description.contains("↗"),
        "遷移の先を出していない: {}",
        card.description
    );
    assert!(
        card.description.contains("の 2 回"),
        "その遷移の回数を出していない: {}",
        card.description
    );
}

/// 投げはガードが外れた話ではない。ガードしていても投げられる。
#[test]
fn a_throw_is_not_a_broken_guard() {
    let events = events_with(
        vec![guard_break(100, "DR", "UR", 0.15)],
        vec![attack(100, AttackAttribute::Throw)],
    );

    assert!(detect_guard_break(&events, 1).is_none());
}

/// 攻撃の素性が読めていれば、その内訳を添える。何に外れているかが
/// 分かると、直す方向が決まる。
#[test]
fn the_attack_attribute_is_reported_when_it_was_read() {
    let events = events_with(
        vec![
            guard_break(100, "DR", "UR", 0.15),
            guard_break(1000, "DR", "UR", 0.12),
        ],
        vec![
            attack(100, AttackAttribute::Lower),
            attack(1000, AttackAttribute::Lower),
        ],
    );

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(
        card.description
            .contains("攻撃属性まで確認できた 2 件のうち、下段が 2 件"),
        "属性の内訳が出ていない: {}",
        card.description
    );
    assert!(
        card.evidence[0].label.contains("下段"),
        "クリップに属性が出ていない: {}",
        card.evidence[0].label
    );
}

/// 素性が読めていなければ、その話は書かない。読めていない情報を
/// 埋めると、確認したことと推測したことが混ざる。
#[test]
fn nothing_is_said_about_an_attack_that_was_not_read() {
    let events = events_with(vec![guard_break(100, "DR", "UR", 0.15)], vec![]);

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(
        !card.description.contains("攻撃属性"),
        "読めていない属性を語っている: {}",
        card.description
    );
}

/// 読み取りが途中で切れた攻撃も使わない。
#[test]
fn an_incomplete_attack_record_is_not_used() {
    let mut events = events_with(
        vec![guard_break(100, "DR", "UR", 0.15)],
        vec![attack(100, AttackAttribute::Lower)],
    );
    events.attack_evidence.damage[0].complete = false;

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(
        !card.description.contains("攻撃属性まで確認できた"),
        "途中で切れた読みを使っている: {}",
        card.description
    );
}

/// 確度の低い攻撃も使わない。
#[test]
fn a_low_confidence_attack_record_is_not_used() {
    let mut events = events_with(
        vec![guard_break(100, "DR", "UR", 0.15)],
        vec![attack(100, AttackAttribute::Lower)],
    );
    events.attack_evidence.damage[0].confidence = EventConfidence::Low;

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(!card.description.contains("攻撃属性まで確認できた"));
}

/// ゲーム内表示のダメージが信用できるときだけ、クリップに数字を出す。
#[test]
fn the_exact_damage_is_shown_only_when_it_is_reliable() {
    let mut events = events_with(
        vec![guard_break(100, "DR", "UR", 0.15)],
        vec![attack(100, AttackAttribute::Lower)],
    );

    let reliable = detect_guard_break(&events, 1).expect("提示される");
    assert!(
        reliable.evidence[0].label.contains("1800"),
        "信用できる数字を出していない: {}",
        reliable.evidence[0].label
    );

    events.attack_evidence.damage[0].hp_consistency = AttackDamageConsistency::Mismatch;
    let unreliable = detect_guard_break(&events, 1).expect("提示される");
    assert!(
        !unreliable.evidence[0].label.contains("1800"),
        "食い違った数字を出している: {}",
        unreliable.evidence[0].label
    );
}

/// 攻撃の記録は被弾の時刻で引く。離れた記録を拾うと、別の攻撃の
/// 素性を書くことになる。
#[test]
fn the_attack_is_matched_by_when_the_hit_landed() {
    let mut events = events_with(
        vec![guard_break(100, "DR", "UR", 0.15)],
        vec![attack(100, AttackAttribute::Lower)],
    );
    events.damage[0].start_frame = 106;
    events.guard_breaks[0].frame = 100;

    let card = detect_guard_break(&events, 1).expect("提示される");

    assert!(
        !card.description.contains("攻撃属性まで確認できた"),
        "離れた攻撃の記録を引いている: {}",
        card.description
    );
}

/// 相手のガード崩れは自分の話ではない。
#[test]
fn the_opponents_broken_guard_is_not_reported() {
    let mut events = events_with(vec![guard_break(100, "DR", "UR", 0.15)], vec![]);
    events.guard_breaks[0].side = 2;

    assert!(detect_guard_break(&events, 1).is_none());
}

/// クリップには外れた方向と失った HP を出す。
#[test]
fn the_clip_says_which_way_the_input_went() {
    let events = events_with(vec![guard_break(100, "DR", "UR", 0.15)], vec![]);

    let card = detect_guard_break(&events, 1).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains("↘"), "{label}");
    assert!(label.contains("↗"), "{label}");
    assert!(label.contains("15"), "失った HP が出ていない: {label}");
}

/// 一度きりと癖で文面を書き分ける。
#[test]
fn the_wording_changes_when_it_becomes_a_habit() {
    let once = events_with(vec![guard_break(100, "DR", "UR", 0.15)], vec![]);
    let habit = events_with(
        vec![
            guard_break(100, "DR", "UR", 0.15),
            guard_break(1000, "DR", "UR", 0.12),
        ],
        vec![],
    );

    let once = detect_guard_break(&once, 1).expect("提示される");
    let habit = detect_guard_break(&habit, 1).expect("提示される");

    assert_eq!(once.id, habit.id);
    assert_ne!(once.title, habit.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, habit.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, habit.practice, "練習方法を書き分けていない");
}
