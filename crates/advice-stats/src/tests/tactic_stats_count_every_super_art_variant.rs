//! SA / CA の使用・結果・使いどころの数え分け。
//!
//! 同じ 1 本でも、コンボに組み込んだのか確反で刺したのか切り返しで
//! 振ったのかで意味が違う。level と CA の別、そして自分側と相手側の別も
//! 混ぜられない。分岐はあるのに、そこへ入る観測がどのテストにも無かった。

use super::support::*;
use crate::match_events::{SuperArtContext, SuperArtEvent, SuperArtOutcome};

fn super_art(frame: u32, side: u8, level: u8, critical_art: bool) -> SuperArtEvent {
    SuperArtEvent {
        side,
        frame,
        gauge_drop_frame: frame,
        level,
        critical_art,
        gauge_before: 1.0,
        gauge_after: 0.0,
        context: SuperArtContext::Unknown,
        outcome: SuperArtOutcome::Unconfirmed,
        contact_frame: Some(frame + 10),
        damage: 0.0,
        ko: false,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 自分の SA は level ごと、CA は別枠で数える。
#[test]
fn own_super_arts_are_counted_by_level_and_critical_art() {
    let mut events = empty_events();
    events.super_arts.push(super_art(100, 1, 1, false));
    events.super_arts.push(super_art(300, 1, 2, false));
    events.super_arts.push(super_art(500, 1, 3, false));
    events.super_arts.push(super_art(700, 1, 3, true));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.sa1_used, 1);
    assert_eq!(stats.sa2_used, 1);
    assert_eq!(stats.sa3_used, 1);
    assert_eq!(stats.ca_used, 1, "CA は SA3 と別に数える");
}

/// 相手の SA も同じ粒度で数える。自分側と混ざると「撃たれた回数」と
/// 「撃った回数」が入れ替わる。
#[test]
fn opponent_super_arts_are_counted_by_level_and_critical_art() {
    let mut events = empty_events();
    events.super_arts.push(super_art(100, 2, 1, false));
    events.super_arts.push(super_art(300, 2, 2, false));
    events.super_arts.push(super_art(500, 2, 3, false));
    events.super_arts.push(super_art(700, 2, 3, true));

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.opponent_sa1_used, 1);
    assert_eq!(stats.opponent_sa2_used, 1);
    assert_eq!(stats.opponent_sa3_used, 1);
    assert_eq!(stats.opponent_ca_used, 1);
    assert_eq!(stats.sa1_used, 0, "相手の使用を自分側へ数えない");
}

/// 自分の SA の結末は、当てた・ガードされた・触れられなかったで分ける。
/// 結末が確定しない観測はどこにも数えない。
#[test]
fn own_super_art_outcomes_are_counted_separately() {
    let mut events = empty_events();
    for (index, outcome) in [
        SuperArtOutcome::Hit,
        SuperArtOutcome::Blocked,
        SuperArtOutcome::NoImmediateContact,
        SuperArtOutcome::Unconfirmed,
    ]
    .into_iter()
    .enumerate()
    {
        events.super_arts.push(SuperArtEvent {
            outcome,
            ..super_art(100 + index as u32 * 200, 1, 1, false)
        });
    }

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.super_hits, 1);
    assert_eq!(stats.super_blocked, 1);
    assert_eq!(stats.super_no_immediate_contact, 1);
    assert_eq!(stats.sa1_used, 4, "結末が不明でも撃ったことは数える");
}

/// 相手の SA の結末も同じ粒度で分ける。
#[test]
fn opponent_super_art_outcomes_are_counted_separately() {
    let mut events = empty_events();
    for (index, outcome) in [
        SuperArtOutcome::Hit,
        SuperArtOutcome::Blocked,
        SuperArtOutcome::NoImmediateContact,
        SuperArtOutcome::Unconfirmed,
    ]
    .into_iter()
    .enumerate()
    {
        events.super_arts.push(SuperArtEvent {
            outcome,
            ..super_art(100 + index as u32 * 200, 2, 1, false)
        });
    }

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.opponent_super_hits, 1);
    assert_eq!(stats.opponent_super_blocked, 1);
    assert_eq!(stats.opponent_super_no_immediate_contact, 1);
    assert_eq!(stats.opponent_sa1_used, 4);
}

/// KO と被確反は、当たり外れとは別に数える。どちらも「その 1 本が試合に
/// 効いたか」を直接示す。
#[test]
fn kos_and_punishes_are_counted_on_both_sides() {
    let mut events = empty_events();
    events.super_arts.push(SuperArtEvent {
        ko: true,
        punished: true,
        ..super_art(100, 1, 3, true)
    });
    events.super_arts.push(SuperArtEvent {
        ko: true,
        punished: true,
        ..super_art(300, 2, 3, true)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.super_kos, 1);
    assert_eq!(stats.super_punished, 1);
    assert_eq!(stats.opponent_super_kos, 1);
    assert_eq!(stats.opponent_super_punished, 1);
}

/// 使いどころは、コンボ・確反・切り返し・中間で分ける。ここが混ざると
/// 「どこで使い切っているか」を指摘できない。
#[test]
fn own_super_art_contexts_are_counted_separately() {
    let mut events = empty_events();
    for (index, context) in [
        SuperArtContext::Combo,
        SuperArtContext::Punish,
        SuperArtContext::DefensiveReversal,
        SuperArtContext::Neutral,
        SuperArtContext::Unknown,
    ]
    .into_iter()
    .enumerate()
    {
        events.super_arts.push(SuperArtEvent {
            context,
            ..super_art(100 + index as u32 * 200, 1, 1, false)
        });
    }

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.super_combo_uses, 1);
    assert_eq!(stats.super_punish_uses, 1);
    assert_eq!(stats.super_reversal_uses, 1);
    assert_eq!(stats.super_neutral_uses, 1);
    assert_eq!(
        stats.super_combo_uses
            + stats.super_punish_uses
            + stats.super_reversal_uses
            + stats.super_neutral_uses,
        4,
        "使いどころ不明は どの欄にも数えない"
    );
}

/// 確信度の低い観測は、使用そのものを数えない。
#[test]
fn a_low_confidence_super_art_is_not_counted_at_all() {
    let mut events = empty_events();
    events.super_arts.push(SuperArtEvent {
        confidence: EventConfidence::Low,
        outcome: SuperArtOutcome::Hit,
        ..super_art(100, 1, 1, false)
    });

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.sa1_used, 0);
    assert_eq!(stats.super_hits, 0);
}
