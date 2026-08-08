use super::support::*;
use crate::match_events::{ThrowActionEvent, ThrowApproach, ThrowOutcome};

#[test]
fn single_punished_throw_whiff_is_an_observation_and_owns_the_big_hit() {
    let mut events = empty_events();
    events.throw_actions.push(whiff(100, 105));
    events.damage.push(damage(130, 170, 0.20));

    let report = detector_test_report(&events, "p1");
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "throw_whiff_punished")
        .expect("punished throw whiff");

    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::High);
    assert_invites_user_review(card);
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].frame, 100);
    assert_eq!(card.evidence[0].end_frame, Some(170));
    assert!(report.cards.iter().all(|card| card.id != "big_hits"));
}

#[test]
fn consecutive_throw_whiffs_before_one_hit_form_one_diagnosis_clip() {
    let mut events = empty_events();
    events.throw_actions = vec![whiff(100, 105), whiff(140, 145)];
    events.damage.push(damage(180, 230, 0.28));

    let report = detector_test_report(&events, "p1");
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "throw_whiff_punished")
        .expect("repeated punished throw whiffs");

    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].frame, 100);
    assert_eq!(card.evidence[0].end_frame, Some(230));
    assert!(card.evidence[0].label.contains("空振り2回後"));
    assert!((card.severity - 0.32).abs() < 0.0001);
}

#[test]
fn unpunished_or_unconfirmed_throw_whiffs_do_not_emit_advice() {
    let mut events = empty_events();
    events.throw_actions.push(whiff(100, 105));
    let mut uncertain = whiff(300, 305);
    uncertain.confidence = EventConfidence::Medium;
    events.throw_actions.push(uncertain);
    events.damage.push(damage(400, 430, 0.20));

    let report = detector_test_report(&events, "p1");
    assert!(report
        .cards
        .iter()
        .all(|card| card.id != "throw_whiff_punished"));
}

#[test]
fn throw_interrupted_by_invincible_has_neutral_advice_and_is_not_a_whiff() {
    let mut events = empty_events();
    events.throw_actions.push(ThrowActionEvent {
        thrower: 1,
        input_frame: 100,
        startup_frame: Some(103),
        active_frame: Some(105),
        outcome: ThrowOutcome::InterruptedByInvincible,
        damage: 0.0,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.damage.push(damage(109, 170, 0.20));

    let report = detector_test_report(&events, "p1");
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "throw_interrupted_by_invincible")
        .expect("無敵技に負けた投げは専用カードへ分ける");

    assert_eq!(card.kind, AdviceKind::Observation);
    assert!(card
        .description
        .contains("投げ間合いの空振りではありません"));
    assert_invites_user_review(card);
    assert!(report
        .cards
        .iter()
        .all(|card| card.id != "throw_whiff_punished" && card.id != "big_hits"));
}

fn whiff(input_frame: u32, active_frame: u32) -> ThrowActionEvent {
    ThrowActionEvent {
        thrower: 1,
        input_frame,
        startup_frame: Some(active_frame - 3),
        active_frame: Some(active_frame),
        outcome: ThrowOutcome::ExecutedWhiff,
        damage: 0.0,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn damage(start_frame: u32, end_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}
