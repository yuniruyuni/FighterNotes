//! バーンアウトと、有利フレーム・不利フレームの使い方の数え分け。
//!
//! バーンアウトは「なぜそうなったか」で直し方が変わる。有利側は続けたか
//! 手放したか、不利側は暴れて負けたかで変わる。分岐はあるのに、そこへ
//! 入る観測がどのテストにも無かった。

use super::support::*;
use crate::match_events::{
    AdvantageOutcome, AdvantageSituationEvent, BurnoutCause, DefensiveActionKind, MinusPressEvent,
    MinusPressOutcome,
};

fn burnout(start_frame: u32, cause: BurnoutCause) -> BurnoutPeriod {
    BurnoutPeriod {
        side: 1,
        start_frame,
        end_frame: start_frame + 120,
        hp_lost: 0.1,
        hp_dealt: 0.05,
        cause,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// バーンアウトは原因ごとに数え分ける。自分から使い切ったのか、ガードを
/// 強いられたのかで、次に直すことが変わる。
#[test]
fn a_burnout_is_counted_by_its_cause() {
    let mut events = empty_events();
    for (index, cause) in [
        BurnoutCause::SelfInitiated,
        BurnoutCause::ForcedByGuard,
        BurnoutCause::Mixed,
        BurnoutCause::Unknown,
    ]
    .into_iter()
    .enumerate()
    {
        events
            .burnouts
            .push(burnout(100 + index as u32 * 300, cause));
    }

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.burnout_count, 4);
    assert_eq!(stats.burnout_self_initiated, 1);
    assert_eq!(stats.burnout_forced, 1);
    assert_eq!(stats.burnout_mixed, 1);
    assert_eq!(stats.burnout_unknown, 1);
}

/// バーンアウト中の収支と長さも積む。長さは秒（60fps）に直す。
#[test]
fn burnout_length_and_hp_swing_accumulate() {
    let mut events = empty_events();
    events
        .burnouts
        .push(burnout(100, BurnoutCause::ForcedByGuard));
    events
        .burnouts
        .push(burnout(500, BurnoutCause::ForcedByGuard));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert!(
        (stats.burnout_seconds - 4.0).abs() < 1e-4,
        "120 フレーム 2 回 = 4 秒, got {}",
        stats.burnout_seconds
    );
    assert!((stats.burnout_hp_lost - 0.2).abs() < 1e-4);
    assert!((stats.burnout_hp_dealt - 0.1).abs() < 1e-4);
}

/// 相手のバーンアウトは自分の統計に入れない。
#[test]
fn the_opponent_burnout_is_not_counted_as_ours() {
    let mut events = empty_events();
    events.burnouts.push(BurnoutPeriod {
        side: 2,
        ..burnout(100, BurnoutCause::SelfInitiated)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.burnout_count, 0);
    assert_eq!(stats.burnout_self_initiated, 0);
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

/// 有利フレームは、続けたか手放したかで分ける。手放した上に取り返された
/// 場面はさらに別に数える。
#[test]
fn an_advantage_situation_is_counted_by_what_followed() {
    let mut events = empty_events();
    events
        .advantage_situations
        .push(advantage(100, Some(110), AdvantageOutcome::Continued));
    events
        .advantage_situations
        .push(advantage(300, None, AdvantageOutcome::Reset));
    events
        .advantage_situations
        .push(advantage(400, None, AdvantageOutcome::Reset));
    events
        .advantage_situations
        .push(advantage(500, None, AdvantageOutcome::TurnLost));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.advantage_opportunities, 4);
    assert_eq!(stats.advantage_continued, 1);
    assert_eq!(stats.advantage_abandoned, 3);
    assert_eq!(
        stats.advantage_turns_lost, 1,
        "手放した上で取り返された場面だけを数える"
    );
}

/// 確信度の足りない有利場面は機会に数えない。分母が水増しされる。
#[test]
fn an_unconfirmed_advantage_situation_is_not_an_opportunity() {
    let mut events = empty_events();
    events.advantage_situations.push(AdvantageSituationEvent {
        confidence: EventConfidence::Medium,
        ..advantage(100, None, AdvantageOutcome::TurnLost)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.advantage_opportunities, 0);
    assert_eq!(stats.advantage_turns_lost, 0);
}

fn minus_press(
    frame: u32,
    action_kind: DefensiveActionKind,
    outcome: MinusPressOutcome,
) -> MinusPressEvent {
    MinusPressEvent {
        side: 1,
        frame,
        minus_frames: 5,
        pressed: String::new(),
        action_kind,
        outcome,
        drop: 0.0,
        confidence: EventConfidence::High,
        source_contact_frame: frame,
        round_no: 1,
    }
}

/// 不利フレームからの暴れは、打撃と投げを分けて数える。負けた回数も
/// それぞれ別に持つ。
#[test]
fn a_challenge_while_minus_is_counted_by_action_and_result() {
    let mut events = empty_events();
    events.presses_while_minus.push(minus_press(
        100,
        DefensiveActionKind::Strike,
        MinusPressOutcome::CounterHit,
    ));
    events.presses_while_minus.push(minus_press(
        300,
        DefensiveActionKind::Strike,
        MinusPressOutcome::Won,
    ));
    events.presses_while_minus.push(minus_press(
        500,
        DefensiveActionKind::Throw,
        MinusPressOutcome::CounterHit,
    ));
    events.presses_while_minus.push(minus_press(
        700,
        DefensiveActionKind::Throw,
        MinusPressOutcome::GotAway,
    ));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.fastest_strike_challenges, 2);
    assert_eq!(stats.fastest_strike_losses, 1);
    assert_eq!(stats.fastest_throw_challenges, 2);
    assert_eq!(stats.fastest_throw_losses, 1);
}
