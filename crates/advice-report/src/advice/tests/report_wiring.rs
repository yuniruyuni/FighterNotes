//! レポートの組み立て結線に対するテスト。
//!
//! この層は、イベント層の出力を欄ごとに詰め替えるだけの層。詰め忘れても
//! 例外は出ず、その欄だけが空のまま返る。空のレポートは「何も起きなかった
//! 試合」と見分けが付かないので、どの欄も必ず埋まることを固定する。

use super::support::*;
use crate::match_events::{
    DamageEvent, DriveImpactEvent, DriveImpactOutcome, EventConfidence, RoundInfo,
};

/// 被弾のある 1 ラウンドの試合。
fn events_with_damage() -> MatchEvents {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 9,
        winner: Some(2),
        p1_hp_end: 0.4,
        p2_hp_end: 1.0,
    }];
    events.damage = vec![DamageEvent {
        victim: 1,
        start_frame: 3,
        pre_freeze_frame: 3,
        end_frame: 5,
        hp_before: 1.0,
        hp_after: 0.4,
        drop: 0.6,
        round_no: 1,
    }];
    events.hp = [vec![1.0; 10], vec![1.0; 10]];
    events
}

/// 被弾はそのままレポートの被弾一覧になる。
#[test]
fn the_damage_events_reach_the_report() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert_eq!(report.damage_taken_events.len(), 1, "被弾を詰めていない");
    assert!((report.damage_taken_events[0].hp_drop - 0.6).abs() < 1e-5);
}

/// ラウンドの数と、ラウンドごとの要約がそれぞれ入る。
#[test]
fn the_rounds_reach_both_the_count_and_the_summaries() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert_eq!(report.rounds_detected, 1);
    assert_eq!(
        report.round_summaries.len(),
        1,
        "ラウンド要約を詰めていない"
    );
    assert_eq!(report.round_summaries[0].won, Some(false));
}

/// 自分の側が変われば、勝敗の見え方も変わる。
#[test]
fn the_side_decides_who_won_the_round() {
    let events = events_with_damage();

    let as_p1 = detector_test_report(&events, "p1");
    let as_p2 = detector_test_report(&events, "p2");

    assert_eq!(as_p1.round_summaries[0].won, Some(false));
    assert_eq!(as_p2.round_summaries[0].won, Some(true));
    assert_eq!(
        as_p2.damage_taken_events.len(),
        0,
        "相手の被弾を自分の被弾に数えている"
    );
}

/// 相手側はカードだけでなく、独立した戦術統計にも渡す。P2 視点で
/// `3 - own` を取り違えると、P1 の DI が相手行動として数えられない。
#[test]
fn the_opponent_side_reaches_tactic_stats() {
    let mut events = empty_events();
    events.drive_impacts.push(DriveImpactEvent {
        side: 1,
        input_frame: 100,
        active_frame: Some(120),
        contact_frame: Some(120),
        outcome: DriveImpactOutcome::Blocked,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let report = detector_test_report(&events, "p2");

    assert_eq!(report.tactic_stats.di_faced, 1);
    assert_eq!(report.tactic_stats.di_blocked, 1);
}

/// 旧 API で渡された自キャラクターは、確反候補を選ぶ検出器まで届く。
#[test]
fn the_legacy_own_character_reaches_advice_detectors() {
    let fixture = advice_detectors::test_support::card_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "punish_missed")
        .expect("確反見逃し fixture");

    let report = detector_test_report_with_character(&fixture.events, "p1", Some("LUKE"));
    let card = report
        .cards
        .iter()
        .find(|card| card.id == fixture.id)
        .expect("確反見逃しカード");

    assert!(
        card.description.contains("威力"),
        "キャラクター別の確反候補が届いていない: {}",
        card.description
    );
}

/// 失った HP は、どこから来たかの内訳にも入る。
#[test]
fn the_damage_reaches_the_breakdown() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert!(
        report.damage_breakdown.total_hp_lost > 0.0,
        "内訳を組み立てていない: {:?}",
        report.damage_breakdown
    );
    assert_eq!(report.damage_breakdown.events.len(), 1);
}

/// 総フレーム数は、観測した最後のフレーム番号の次。0 始まりの番号を
/// そのまま数にすると 1 フレーム足りない。
#[test]
fn the_total_frames_counts_from_one() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert_eq!(report.total_frames, 10, "最後のフレームを数えていない");
}

/// 要約は指摘とラウンド数と被弾数から書く。空文字のまま返さない。
#[test]
fn the_summary_is_written_from_what_was_found() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert!(!report.summary.is_empty(), "要約が空のまま");
    assert!(
        report.summary.contains("1ラウンド"),
        "ラウンド数が要約に入っていない: {}",
        report.summary
    );
    assert!(
        report.summary.contains("被弾 1 件"),
        "被弾数が要約に入っていない: {}",
        report.summary
    );
}

/// 読み取りの網羅度は必ず測る。測っていないレポートは、指摘が無いのか
/// 読めなかったのか区別できない。
#[test]
fn the_coverage_is_always_measured() {
    let report = detector_test_report(&events_with_damage(), "p1");

    assert!(report.coverage.match_frames > 0, "網羅度を測っていない");
    assert!(report.coverage.availability.is_some());
}

/// 解析器のビルド ID は、版数と改訂の組。同じ ruleset でも配布物を
/// 特定できるようにする。
#[test]
fn the_build_id_carries_the_package_version() {
    let report = detector_test_report(&empty_events(), "p1");

    let (version, revision) = report
        .analyzer_build_id
        .split_once('+')
        .expect("版数と改訂の組");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    assert!(!revision.is_empty(), "改訂が空");
}

/// 何も起きていない試合でも、欄は形として揃える。
#[test]
fn an_empty_match_still_produces_a_shaped_report() {
    let mut events = empty_events();
    events.rounds.clear();
    let report = detector_test_report(&events, "p1");

    assert_eq!(report.rounds_detected, 0);
    assert!(report.damage_taken_events.is_empty());
    assert!(report.round_summaries.is_empty());
    assert!(!report.summary.is_empty(), "要約だけは必ず書く");
    assert_ne!(report.ruleset_version, 0);
}
