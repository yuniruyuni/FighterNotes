//! Drive Impact / Drive Rush の結果ごとの数え分け。
//!
//! 履歴集計に載る戦術統計は、結果の内訳まで含めて意味を持つ。ガードした
//! のかパリィしたのか空振らせたのかで、次に直すべきことが変わる。分岐は
//! あるのに、その分岐へ入る観測がどのテストにも無い状態だった。

use super::support::*;
use crate::match_events::{DriveImpactEvent, DriveImpactOutcome, DriveRushEvent, DriveRushOutcome};

fn impact(input_frame: u32, outcome: DriveImpactOutcome) -> DriveImpactEvent {
    DriveImpactEvent {
        side: 2,
        input_frame,
        active_frame: Some(input_frame + 20),
        contact_frame: Some(input_frame + 20),
        outcome,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 相手の Drive Impact は、確定した結果ごとに別々の欄へ数える。
#[test]
fn a_faced_drive_impact_is_counted_by_its_outcome() {
    let mut events = empty_events();
    for (index, outcome) in [
        DriveImpactOutcome::Countered,
        DriveImpactOutcome::Blocked,
        DriveImpactOutcome::Parried,
        DriveImpactOutcome::Hit,
        DriveImpactOutcome::Whiffed,
    ]
    .into_iter()
    .enumerate()
    {
        events
            .drive_impacts
            .push(impact(100 + index as u32 * 100, outcome));
    }

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.di_faced, 5, "確定した DI は全て機会として数える");
    assert_eq!(stats.di_returned, 1);
    assert_eq!(stats.di_blocked, 1);
    assert_eq!(stats.di_parried, 1);
    assert_eq!(stats.di_hit, 1);
    assert_eq!(stats.di_avoided, 1);
    assert_eq!(stats.di_unconfirmed, 0);
}

/// 結果を確定できなかった DI は機会に数えず、未確定として別に持つ。
/// 混ぜると「返せた割合」の分母が水増しされる。
#[test]
fn an_unconfirmed_drive_impact_is_kept_out_of_the_denominator() {
    let mut events = empty_events();
    events.drive_impacts.push(DriveImpactEvent {
        confidence: EventConfidence::Medium,
        ..impact(100, DriveImpactOutcome::Blocked)
    });
    // 結果そのものが未確定な場合は、確定扱いでも未確定欄へ入る。
    events
        .drive_impacts
        .push(impact(300, DriveImpactOutcome::Unconfirmed));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.di_faced, 1, "確定扱いの 1 件だけが機会になる");
    assert_eq!(stats.di_unconfirmed, 2);
    assert_eq!(stats.di_blocked, 0, "未確定の観測を結果へ振り分けない");
}

/// 自分側の DI は相手側とは別の欄へ数える。側の取り違えは
/// 「返された」と「返した」を入れ替えるので、必ず区別する。
#[test]
fn own_drive_impacts_are_counted_apart_from_the_opponent_ones() {
    let mut events = empty_events();
    events.drive_impacts.push(DriveImpactEvent {
        side: 1,
        ..impact(100, DriveImpactOutcome::Hit)
    });
    events
        .drive_impacts
        .push(impact(300, DriveImpactOutcome::Hit));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.own_di_hit, 1);
    assert_eq!(stats.di_faced, 1);
    assert_eq!(stats.di_hit, 1);
}

fn rush(frame: u32, outcome: DriveRushOutcome) -> DriveRushEvent {
    DriveRushEvent {
        side: 2,
        frame,
        raw: true,
        outcome,
        contact_frame: Some(frame + 20),
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 生ラッシュも結果ごとに数え分ける。止められたのか通されたのかが
/// 分からないと、置き技を増やすべきかどうかを言えない。
#[test]
fn a_raw_drive_rush_is_counted_by_its_outcome() {
    let mut events = empty_events();
    // 止めた形は三通りあり、いずれも「凌いだ」として一つに畳む。
    for (index, outcome) in [
        DriveRushOutcome::Blocked,
        DriveRushOutcome::Stopped,
        DriveRushOutcome::NoContact,
    ]
    .into_iter()
    .enumerate()
    {
        events
            .drive_rushes
            .push(rush(100 + index as u32 * 100, outcome));
    }
    events.drive_rushes.push(rush(500, DriveRushOutcome::Hit));
    events
        .drive_rushes
        .push(rush(700, DriveRushOutcome::Unconfirmed));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(
        stats.raw_drive_rushes_faced, 5,
        "結果が未確定でも観測できた生ラッシュは機会に数える"
    );
    assert_eq!(stats.raw_drive_rushes_defended, 3);
    assert_eq!(stats.raw_drive_rushes_hit, 1);
    assert_eq!(stats.raw_drive_rushes_unconfirmed, 1);
}

/// 確信度の足りない生ラッシュは未確定へ回す。
#[test]
fn an_unconfirmed_raw_drive_rush_is_kept_out_of_the_denominator() {
    let mut events = empty_events();
    events.drive_rushes.push(DriveRushEvent {
        confidence: EventConfidence::Medium,
        ..rush(100, DriveRushOutcome::Hit)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.raw_drive_rushes_faced, 0);
    assert_eq!(stats.raw_drive_rushes_unconfirmed, 1);
    assert_eq!(stats.raw_drive_rushes_hit, 0);
}

/// 生でないラッシュ（キャンセルラッシュ）は、生ラッシュの統計に入れない。
/// 対処すべき場面が別物になる。
#[test]
fn a_cancelled_drive_rush_is_not_a_raw_one() {
    let mut events = empty_events();
    events.drive_rushes.push(DriveRushEvent {
        raw: false,
        ..rush(100, DriveRushOutcome::Hit)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.raw_drive_rushes_faced, 0);
    assert_eq!(stats.raw_drive_rushes_unconfirmed, 0);
}

/// 未確定の先頭要素を飛ばしても、同じ列の後続イベントは集計を続ける。
/// `continue` を `break` にすると、三種類とも後続の確定結果が消える。
#[test]
fn an_unconfirmed_event_does_not_hide_later_drive_events() {
    let mut events = empty_events();

    let mut uncertain_own_impact = impact(100, DriveImpactOutcome::Hit);
    uncertain_own_impact.side = 1;
    uncertain_own_impact.confidence = EventConfidence::Medium;
    let mut confirmed_own_impact = impact(200, DriveImpactOutcome::Hit);
    confirmed_own_impact.side = 1;
    events.drive_impacts = vec![uncertain_own_impact, confirmed_own_impact];

    let mut uncertain_own_rush = rush(300, DriveRushOutcome::Hit);
    uncertain_own_rush.side = 1;
    uncertain_own_rush.confidence = EventConfidence::Medium;
    let mut confirmed_own_rush = rush(400, DriveRushOutcome::Hit);
    confirmed_own_rush.side = 1;
    let mut uncertain_faced_rush = rush(500, DriveRushOutcome::Hit);
    uncertain_faced_rush.confidence = EventConfidence::Medium;
    let confirmed_faced_rush = rush(600, DriveRushOutcome::Hit);
    events.drive_rushes = vec![
        uncertain_own_rush,
        confirmed_own_rush,
        uncertain_faced_rush,
        confirmed_faced_rush,
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.own_di_hit, 1);
    assert_eq!(stats.own_raw_drive_rush_hits, 1);
    assert_eq!(stats.raw_drive_rushes_hit, 1);
}
