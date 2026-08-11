//! 開幕の被弾と、低い補正率での SA 投入に対するテスト。
//!
//! どちらも断定しない一覧。開幕に被弾したこと自体は、最初の行動が
//! 同じだったかまで確かめられていないので癖とは呼べない。低い補正率での
//! SA も、KO・運び・起き攻めのどれかに効いていたなら正しい判断でありうる。
//!
//! 一覧に何を並べるかだけが仕事なので、並べるものを取り違えると、
//! 見返す時間がそのまま無駄になる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    AttackDamageConsistency, DamageAttackEvidence, DamageEvent, EventConfidence, MatchEvents,
    SuperArtAttackEvidence, SuperArtContext, SuperArtEvent, SuperArtOutcome,
};
use crate::{AdviceKind, RoundSummary};

// ── 開幕の被弾 ───────────────────────────────────────────────────────────

fn round(round_no: u32, start_frame: u32, early_hit: bool) -> RoundSummary {
    RoundSummary {
        round_no,
        start_frame,
        end_frame: start_frame + 3_000,
        won: Some(false),
        own_hp_end: 0.0,
        opp_hp_end: 0.5,
        own_hp_lost: 1.0,
        opp_hp_lost: 0.5,
        own_hits_taken: 4,
        early_hit,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    }
}

fn taken(round_no: u32, start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame.saturating_sub(10),
        end_frame: start_frame + 40,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no,
    }
}

fn events_with_damage(damage: Vec<DamageEvent>) -> MatchEvents {
    MatchEvents {
        damage,
        ..empty_events()
    }
}

/// 開幕に被弾したラウンドが無ければ何も出さない。
#[test]
fn nothing_is_reported_without_an_early_hit() {
    let rounds = vec![round(1, 0, false), round(2, 4_000, false)];

    assert!(detect_early_hits(&empty_events(), &rounds, 1).is_none());
}

/// 被弾したラウンドを並べる。癖とまでは言わない。
#[test]
fn the_rounds_are_listed_without_calling_it_a_habit() {
    let rounds = vec![round(1, 0, true), round(2, 4_000, false)];
    let events = events_with_damage(vec![taken(1, 100, 0.12)]);

    let card = detect_early_hits(&events, &rounds, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.severity - 0.05).abs() < 1e-6);
    assert!(
        card.description.contains("2 ラウンド中 1 ラウンド"),
        "全ラウンドに対する数を出していない: {}",
        card.description
    );
    assert!(
        card.description.contains("断定"),
        "確かめていないことを決めている: {}",
        card.description
    );
}

/// 開幕の被弾は、そのラウンドの頭から数えた時間で決まる。ラウンドの
/// 開始位置を無視すると、二本目以降が全部「開幕」になる。
#[test]
fn the_early_window_is_measured_from_the_rounds_own_start() {
    let rounds = vec![round(2, 4_000, true)];
    let inside = events_with_damage(vec![taken(2, 4_179, 0.12)]);
    let outside = events_with_damage(vec![taken(2, 4_180, 0.12)]);

    let inside = detect_early_hits(&inside, &rounds, 1).expect("提示される");
    let outside = detect_early_hits(&outside, &rounds, 1).expect("提示される");

    assert_eq!(inside.hp_lost, Some(0.12), "窓の内側の被弾を落としている");
    assert_eq!(outside.hp_lost, Some(0.0), "窓の外の被弾を数えている");
}

/// 別のラウンドの被弾は数えない。
#[test]
fn a_hit_from_another_round_is_not_counted() {
    let rounds = vec![round(2, 4_000, true)];
    let events = events_with_damage(vec![taken(1, 100, 0.12)]);

    let card = detect_early_hits(&events, &rounds, 1).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.0));
}

/// 相手が受けた被弾は自分の話ではない。
#[test]
fn a_hit_the_opponent_took_is_not_counted() {
    let rounds = vec![round(1, 0, true)];
    let mut events = events_with_damage(vec![taken(1, 100, 0.12)]);
    events.damage[0].victim = 2;

    let card = detect_early_hits(&events, &rounds, 1).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.0));
}

/// 被弾を特定できたラウンドは、演出の前から被弾の終わりまで映す。
#[test]
fn a_located_hit_gets_a_clip_around_it() {
    let rounds = vec![round(1, 0, true)];
    let events = events_with_damage(vec![taken(1, 100, 0.12)]);

    let card = detect_early_hits(&events, &rounds, 1).expect("提示される");
    let clip = &card.evidence[0];

    assert_eq!(clip.frame, 90, "演出の前から始まっていない");
    assert_eq!(clip.end_frame, Some(140));
    assert!(
        clip.label.contains("12"),
        "失った HP が出ていない: {}",
        clip.label
    );
}

/// 被弾を特定できなければ、ラウンドの頭から見せる。数字は書かない。
#[test]
fn a_round_without_a_located_hit_starts_from_its_beginning() {
    let rounds = vec![round(2, 4_000, true)];

    let card = detect_early_hits(&empty_events(), &rounds, 1).expect("提示される");
    let clip = &card.evidence[0];

    assert_eq!(clip.frame, 4_000);
    assert_eq!(clip.end_frame, None);
    assert!(!clip.label.is_empty());
}

/// 開幕に被弾したラウンドが多いほど重く扱う。
#[test]
fn more_early_hits_weigh_more() {
    let once = vec![round(1, 0, true), round(2, 4_000, false)];
    let twice = vec![round(1, 0, true), round(2, 4_000, true)];
    let events = events_with_damage(vec![taken(1, 100, 0.12), taken(2, 4_100, 0.12)]);

    let once = detect_early_hits(&events, &once, 1).expect("提示される");
    let twice = detect_early_hits(&events, &twice, 1).expect("提示される");

    assert!(twice.severity > once.severity, "回数が重みに効いていない");
    assert_eq!(twice.evidence.len(), 2);
}

// ── 低い補正率での SA 投入 ───────────────────────────────────────────────

fn super_use(frame: u32, level: u8) -> SuperArtEvent {
    SuperArtEvent {
        side: 1,
        frame,
        gauge_drop_frame: frame + 5,
        level,
        critical_art: false,
        gauge_before: 3.0,
        gauge_after: 3.0 - level as f32,
        context: SuperArtContext::Combo,
        outcome: SuperArtOutcome::Hit,
        contact_frame: Some(frame + 20),
        damage: 0.15,
        ko: false,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn super_reading(frame: u32, entry: u32, marginal: u32) -> SuperArtAttackEvidence {
    SuperArtAttackEvidence {
        side: 1,
        super_frame: frame,
        combo_damage: 3600,
        marginal_damage: Some(marginal),
        entry_scaling_percent: Some(entry),
        final_scaling_percent: 30,
        confidence: EventConfidence::High,
    }
}

/// SA の対象になった被弾と、その読み取り。
fn linked_damage(frame: u32) -> (DamageEvent, DamageAttackEvidence) {
    let damage = DamageEvent {
        victim: 2,
        start_frame: frame + 20,
        pre_freeze_frame: frame,
        end_frame: frame + 90,
        hp_before: 1.0,
        hp_after: 0.7,
        drop: 0.3,
        round_no: 1,
    };
    let evidence = DamageAttackEvidence {
        victim: 2,
        attacker: 1,
        damage_start_frame: damage.start_frame,
        sequence_start_frame: damage.start_frame,
        sequence_end_frame: damage.end_frame,
        combo_damage: 3600,
        sequence_count: 1,
        final_scaling_percent: 30,
        starter_attribute: None,
        final_attribute: crate::attack_info::AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    };
    (damage, evidence)
}

/// 低い補正率で SA を投入した一場面。
fn one_low_entry(entry: u32) -> MatchEvents {
    let mut events = empty_events();
    let (damage, evidence) = linked_damage(1000);
    events.super_arts = vec![super_use(1000, 3)];
    events.attack_evidence.super_arts = vec![super_reading(1000, entry, 400)];
    events.damage = vec![damage];
    events.attack_evidence.damage = vec![evidence];
    events
}

/// SA を使っていなければ何も出さない。
#[test]
fn nothing_is_reported_without_a_super() {
    assert!(detect_low_scaling_super(&empty_events(), 1).is_none());
}

/// 補正率が十分なら指摘しない。
#[test]
fn a_super_entered_at_a_healthy_scaling_is_not_reported() {
    let at_the_edge = one_low_entry(50);
    let above = one_low_entry(51);

    assert!(
        detect_low_scaling_super(&at_the_edge, 1).is_some(),
        "閾値ちょうどの補正率を見ていない"
    );
    assert!(
        detect_low_scaling_super(&above, 1).is_none(),
        "十分な補正率まで指摘している"
    );
}

/// KO した SA は、補正率が低くても目的を果たしている。
#[test]
fn a_super_that_finished_the_round_is_not_questioned() {
    let mut events = one_low_entry(30);
    events.super_arts[0].ko = true;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// 相手の SA は自分の話ではない。
#[test]
fn the_opponents_super_is_not_yours() {
    let mut events = one_low_entry(30);
    events.super_arts[0].side = 2;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// 表示を読み切れていない SA は使わない。補正率の数字が信用できない
/// まま「低かった」とは言えない。
#[test]
fn a_super_whose_reading_is_not_reliable_is_not_used() {
    let mut events = one_low_entry(30);
    events.attack_evidence.super_arts[0].confidence = EventConfidence::Medium;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// 対象の被弾側の読みが HP と食い違っていれば使わない。
#[test]
fn a_reading_that_disagrees_with_the_health_is_not_used() {
    let mut events = one_low_entry(30);
    events.attack_evidence.damage[0].hp_consistency = AttackDamageConsistency::Mismatch;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// 補正率が読めていなければ判断しない。
#[test]
fn a_super_without_an_entry_scaling_is_not_judged() {
    let mut events = one_low_entry(30);
    events.attack_evidence.super_arts[0].entry_scaling_percent = None;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// SA 以降に増えたダメージが読めていなければ判断しない。使う価値が
/// あったかどうかは、その数字が無いと語れない。
#[test]
fn a_super_without_a_marginal_damage_is_not_judged() {
    let mut events = one_low_entry(30);
    events.attack_evidence.super_arts[0].marginal_damage = None;

    assert!(detect_low_scaling_super(&events, 1).is_none());
}

/// 並べるだけで断定しない。残り体力や画面位置は見ていない。
#[test]
fn the_super_card_lists_without_judging() {
    let card = detect_low_scaling_super(&one_low_entry(30), 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::Medium);
    assert_eq!(card.hp_lost, None, "機会の損失を被ダメとして出している");
    assert!((card.severity - 0.03).abs() < 1e-6);
    assert!(
        card.description.contains("断定"),
        "見ていないことを決めている: {}",
        card.description
    );
}

/// SA 以降に増えた分を合計して出す。使った価値を測る唯一の数字。
#[test]
fn the_damage_the_super_added_is_summed() {
    let mut events = one_low_entry(30);
    let (damage, evidence) = linked_damage(3000);
    events.super_arts.push(super_use(3000, 2));
    events
        .attack_evidence
        .super_arts
        .push(super_reading(3000, 25, 250));
    events.damage.push(damage);
    events.attack_evidence.damage.push(evidence);

    let card = detect_low_scaling_super(&events, 1).expect("提示される");

    assert!(
        card.description.contains("合計 650"),
        "増えた分を合計していない: {}",
        card.description
    );
    assert_eq!(card.evidence.len(), 2);
    assert!((card.severity - 0.06).abs() < 1e-6);
}

/// クリップには使った SA と、投入時の補正率、増えた分を出す。
#[test]
fn the_clip_says_which_super_and_at_what_scaling() {
    let card = detect_low_scaling_super(&one_low_entry(30), 1).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains("SA3"), "使った SA が出ていない: {label}");
    assert!(
        label.contains("30%補正"),
        "投入時の補正率が出ていない: {label}"
    );
    assert!(label.contains("+400"), "増えた分が出ていない: {label}");
}

/// CA はレベルではなく CA として出す。
#[test]
fn a_critical_art_is_named_as_such() {
    let mut events = one_low_entry(30);
    events.super_arts[0].critical_art = true;

    let card = detect_low_scaling_super(&events, 1).expect("提示される");

    assert!(
        card.evidence[0].label.contains("CA"),
        "CA と書いていない: {}",
        card.evidence[0].label
    );
}
