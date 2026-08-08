use super::support::*;
use crate::match_events::{RoundInfo, ThrowActionEvent, ThrowOutcome};

fn throw(frame: u32, outcome: ThrowOutcome, thrower: u8) -> ThrowActionEvent {
    ThrowActionEvent {
        thrower,
        input_frame: frame,
        startup_frame: Some(frame),
        active_frame: Some(frame + 5),
        outcome,
        damage: if outcome == ThrowOutcome::Hit {
            0.12
        } else {
            0.0
        },
        approach: Default::default(),
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn events_with(throws: Vec<ThrowActionEvent>) -> MatchEvents {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 5_999,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    events.throw_actions = throws;
    events
}

/// 抜けと被投げを、守る機会という同じ分母の上で数える。
#[test]
fn throw_defence_separates_techs_from_throws_taken() {
    let events = events_with(vec![
        throw(100, ThrowOutcome::Hit, 2),
        throw(400, ThrowOutcome::Teched, 2),
        throw(700, ThrowOutcome::Teched, 2),
        throw(1000, ThrowOutcome::InterruptedByInvincible, 2),
    ]);

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.throws_faced, 4);
    assert_eq!(stats.throws_taken, 1);
    assert_eq!(stats.throws_teched, 2);
    assert_eq!(stats.throws_reversal_escaped, 1);
}

/// 届かない位置で振られた投げは守る機会ではない。分母へ入れると抜け率が
/// 実際より低く出る。
#[test]
fn a_throw_that_never_reached_is_not_a_chance_to_defend() {
    let events = events_with(vec![
        throw(100, ThrowOutcome::ExecutedWhiff, 2),
        throw(400, ThrowOutcome::Unconfirmed, 2),
        throw(700, ThrowOutcome::Teched, 2),
    ]);

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.throws_faced, 1);
    assert_eq!(stats.throws_teched, 1);
    assert_eq!(stats.throws_taken, 0);
}

/// 自分が振った投げを、守る側の集計に混ぜない。
#[test]
fn our_own_throws_are_not_counted_as_defence() {
    let events = events_with(vec![
        throw(100, ThrowOutcome::Hit, 1),
        throw(400, ThrowOutcome::Teched, 1),
        throw(700, ThrowOutcome::Hit, 2),
    ]);

    let stats = build_tactic_stats(&[], &events, 1, 2);

    assert_eq!(stats.throws_faced, 1);
    assert_eq!(stats.throws_taken, 1);
    assert_eq!(stats.throws_teched, 0);
}

/// 確度の低い観測は分母にも分子にも入れない。
#[test]
fn low_confidence_throws_are_ignored() {
    let mut throws = vec![
        throw(100, ThrowOutcome::Hit, 2),
        throw(400, ThrowOutcome::Teched, 2),
    ];
    for event in throws.iter_mut() {
        event.confidence = EventConfidence::Medium;
    }

    let stats = build_tactic_stats(&[], &events_with(throws), 1, 2);

    assert_eq!(stats.throws_faced, 0);
}
