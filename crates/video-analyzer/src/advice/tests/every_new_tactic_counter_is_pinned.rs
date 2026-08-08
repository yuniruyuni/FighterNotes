use super::support::*;
use crate::match_events::{
    DriveImpactEvent, DriveImpactOutcome, DriveRushEvent, DriveRushOutcome, KnockdownEvent,
    MinusPressOutcome, OkizemeOutcome, RoundInfo, WhiffEvent, WhiffOutcome,
};

/// 各カウンタが「到達しさえすれば数える」ものであることを、区別できる件数で
/// 固定する。同じ値を並べると、集計先を取り違えても気付けない。
fn round() -> RoundInfo {
    RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 5_999,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }
}

fn impact(frame: u32, side: u8, outcome: DriveImpactOutcome) -> DriveImpactEvent {
    DriveImpactEvent {
        side,
        input_frame: frame,
        active_frame: Some(frame + 10),
        contact_frame: Some(frame + 12),
        outcome,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn rush(frame: u32, side: u8, outcome: DriveRushOutcome) -> DriveRushEvent {
    DriveRushEvent {
        side,
        frame,
        raw: true,
        outcome,
        contact_frame: Some(frame + 12),
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn knockdown(frame: u32, attacker: u8, okizeme: OkizemeOutcome) -> KnockdownEvent {
    KnockdownEvent {
        side: 3 - attacker,
        attacker,
        frame,
        wakeup_frame: frame + 100,
        setup_frames: 40,
        okizeme,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn whiff(frame: u32, side: u8, outcome: WhiffOutcome) -> WhiffEvent {
    WhiffEvent {
        side,
        frame,
        end_frame: frame + 8,
        outcome,
        drop: 0.0,
        punished_frame: None,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 自分の Drive Impact の結末を、それぞれ別のカウンタへ入れる。
#[test]
fn each_own_drive_impact_outcome_lands_in_its_own_counter() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.drive_impacts = vec![
        impact(100, 1, DriveImpactOutcome::Hit),
        impact(200, 1, DriveImpactOutcome::Blocked),
        impact(300, 1, DriveImpactOutcome::Blocked),
        impact(400, 1, DriveImpactOutcome::Parried),
        impact(500, 1, DriveImpactOutcome::Parried),
        impact(600, 1, DriveImpactOutcome::Parried),
        impact(700, 1, DriveImpactOutcome::Countered),
        impact(800, 1, DriveImpactOutcome::Whiffed),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_used, 8);
    assert_eq!(stats.own_di_hit, 1);
    assert_eq!(stats.own_di_blocked, 2);
    assert_eq!(stats.own_di_parried, 3);
    assert_eq!(stats.own_di_countered, 1);
    assert_eq!(stats.own_di_whiffed, 1);
}

/// 確度の足りない DI は結末別ではなく未確認として数える。
#[test]
fn an_unconfirmed_drive_impact_is_not_given_an_outcome() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    let mut unconfirmed = impact(100, 1, DriveImpactOutcome::Hit);
    unconfirmed.confidence = EventConfidence::Medium;
    events.drive_impacts = vec![unconfirmed];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_used, 0);
    assert_eq!(stats.own_di_hit, 0);
    assert_eq!(stats.own_di_unconfirmed, 1);
}

/// 生ラッシュは通ったものと対処されたものに分ける。
#[test]
fn own_raw_drive_rush_outcomes_split_into_hits_and_defended() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.drive_rushes = vec![
        rush(100, 1, DriveRushOutcome::Hit),
        rush(200, 1, DriveRushOutcome::Blocked),
        rush(300, 1, DriveRushOutcome::Stopped),
        rush(400, 1, DriveRushOutcome::NoContact),
        // 相手の生ラッシュは自分側の集計へ混ぜない。
        rush(500, 2, DriveRushOutcome::Hit),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_raw_drive_rushes, 4);
    assert_eq!(stats.own_raw_drive_rush_hits, 1);
    assert_eq!(stats.own_raw_drive_rush_defended, 3);
    assert_eq!(stats.raw_drive_rushes_faced, 1);
}

/// 起き攻めの結末を、取ったダウンと取られたダウンで別に数える。
#[test]
fn knockdowns_are_counted_from_both_sides() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.knockdowns = vec![
        knockdown(100, 1, OkizemeOutcome::Meaty),
        knockdown(500, 1, OkizemeOutcome::Pressured),
        knockdown(900, 1, OkizemeOutcome::Pressured),
        knockdown(1300, 1, OkizemeOutcome::Neutral),
        knockdown(1700, 1, OkizemeOutcome::Neutral),
        knockdown(2100, 1, OkizemeOutcome::Neutral),
        // 取られたダウン。重ねられたかだけを見る。
        knockdown(2500, 2, OkizemeOutcome::Meaty),
        knockdown(2900, 2, OkizemeOutcome::Neutral),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.knockdowns_scored, 6);
    assert_eq!(stats.okizeme_meaty, 1);
    assert_eq!(stats.okizeme_pressured, 2);
    assert_eq!(stats.okizeme_neutral, 3);
    assert_eq!(stats.knockdowns_taken, 2);
    assert_eq!(stats.okizeme_faced_meaty, 1);
}

/// 空振りと被反撃を、自分側と相手側で別に数える。
#[test]
fn whiffs_are_counted_per_side() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.whiffs = vec![
        whiff(100, 1, WhiffOutcome::Punished),
        whiff(300, 1, WhiffOutcome::Unpunished),
        whiff(500, 1, WhiffOutcome::Unpunished),
        whiff(700, 2, WhiffOutcome::Punished),
        whiff(900, 2, WhiffOutcome::Punished),
        whiff(1100, 2, WhiffOutcome::Unpunished),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.whiffs, 3);
    assert_eq!(stats.whiffs_punished, 1);
    assert_eq!(stats.opponent_whiffs, 3);
    assert_eq!(stats.opponent_whiffs_punished, 2);
}

/// ラウンドの外側にあるイベントはどのカウンタにも入れない。
/// round 判定を落とすと、リプレイ冒頭の誤検出まで数えてしまう。
#[test]
fn events_outside_a_round_are_not_counted() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    // round_no は 1 だが、frame が確定ラウンドの外にある。
    let mut stray_impact = impact(9_000, 1, DriveImpactOutcome::Hit);
    stray_impact.round_no = 1;
    events.drive_impacts = vec![stray_impact];
    let mut stray_down = knockdown(9_000, 1, OkizemeOutcome::Meaty);
    stray_down.round_no = 1;
    events.knockdowns = vec![stray_down];
    let mut stray_whiff = whiff(9_000, 1, WhiffOutcome::Punished);
    stray_whiff.round_no = 1;
    events.whiffs = vec![stray_whiff];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_used, 0);
    assert_eq!(stats.knockdowns_scored, 0);
    assert_eq!(stats.whiffs, 0);
}

/// 結末そのものが未確定の DI は、確度が高くても結末別には数えない。
/// 確度と結末は別の軸で、片方だけで判断すると取り違える。
#[test]
fn an_outcome_of_unconfirmed_is_counted_apart_from_low_confidence() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.drive_impacts = vec![impact(100, 1, DriveImpactOutcome::Unconfirmed)];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.own_di_unconfirmed, 1);
    assert_eq!(stats.own_di_hit, 0);
}

/// 起き上がりに重ねられた数は Meaty だけを数える。取られたダウンの多くが
/// Meaty でない構成にして、条件を反転させたら合わないようにする。
#[test]
fn only_meaty_counts_as_being_met_on_wakeup() {
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.knockdowns = vec![
        knockdown(100, 2, OkizemeOutcome::Meaty),
        knockdown(500, 2, OkizemeOutcome::Neutral),
        knockdown(900, 2, OkizemeOutcome::Pressured),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.knockdowns_taken, 3);
    assert_eq!(stats.okizeme_faced_meaty, 1);
}

/// 偏りは割合として出す。件数だけでは「4回中3回」と「40回中3回」が
/// 区別できない。
#[test]
fn the_option_bias_is_reported_as_a_percentage() {
    use crate::match_events::{DefensiveActionKind, MinusPressEvent, MinusSituationEvent};
    let mut events = empty_events();
    events.rounds = vec![round()];
    events.presses_while_minus = (0..3)
        .map(|index| MinusPressEvent {
            side: 1,
            frame: 100 + index * 100,
            minus_frames: 5,
            pressed: "弱".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::GotAway,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 90 + index * 100,
            round_no: 1,
        })
        .collect();
    events.minus_situations = vec![MinusSituationEvent {
        side: 1,
        frame: 500,
        minus_frames: 5,
        fastest_action: None,
        action_frame: None,
        pressed: String::new(),
        outcome: None,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: 490,
        round_no: 1,
    }];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.disadvantage_decisions, 4);
    assert_eq!(stats.disadvantage_top_option_percent, 75);
}
