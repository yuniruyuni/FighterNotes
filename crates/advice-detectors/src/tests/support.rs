//! 検出器のテストで使う観測列。
//!
//! 器（何も起きていない 1 ラウンド）はイベント層の test-support から借り、
//! ここへ主題のイベントだけを足す。

pub(super) use crate::detectors::*;
pub(super) use crate::match_events::*;
pub(super) use crate::*;
pub(super) use match_event_layer::test_support::empty_events;

pub(super) fn assert_invites_user_review(card: &AdviceCard) {
    assert_eq!(
        OBSERVATION_REVIEW_CAVEAT,
        "断定できませんが、検討の対象にしてもよいかもしれません"
    );
    assert!(
        card.description.contains(OBSERVATION_REVIEW_CAVEAT),
        "確認場面が利用者の検討を促していない: {}",
        card.description
    );
}

pub(super) fn basic_mashing_events() -> MatchEvents {
    use crate::match_events::InputSegment;

    let mut events = empty_events();
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 0.9,
        hp_after: 0.78,
        drop: 0.12,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1200,
        pre_freeze_frame: 1200,
        end_frame: 1220,
        hp_before: 0.78,
        hp_after: 0.66,
        drop: 0.12,
        round_no: 1,
    });
    let press = |start_frame| InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    events.segments[0] = vec![press(990), press(1190)];
    events
}
