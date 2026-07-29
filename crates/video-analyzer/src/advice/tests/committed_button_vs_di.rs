use super::support::*;
use crate::match_events::{
    DriveImpactEvent, DriveImpactOutcome, EventConfidence, InputSegment, MeterState,
};

#[test]
fn confirmed_normal_caught_by_opponent_di_has_its_own_review_card() {
    let mut events = basic_mashing_events();
    let damage = events
        .damage
        .iter_mut()
        .find(|damage| damage.start_frame == 1000)
        .unwrap();
    damage.end_frame = 1120;
    damage.hp_after = 0.66;
    damage.drop = 0.24;
    events.segments[0] = vec![InputSegment {
        start_frame: 990,
        end_frame: 994,
        dir: "N".to_string(),
        badges: vec!["強K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }];
    events.drive_impacts.push(DriveImpactEvent {
        side: 2,
        input_frame: 970,
        active_frame: Some(997),
        contact_frame: Some(1000),
        outcome: DriveImpactOutcome::Hit,
        damage: 0.24,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.meter_state = [vec![MeterState::Free; 1300], vec![MeterState::Free; 1300]];
    for state in &mut events.meter_state[0][990..997] {
        *state = MeterState::Startup;
    }
    for state in &mut events.meter_state[0][997..1000] {
        *state = MeterState::Active;
    }
    events.meter_state[0][1000] = MeterState::Recovery;
    events.meter_confidence = [vec![1.0; 1300], vec![1.0; 1300]];

    let card = detect_committed_button_vs_di(&events, 1, 0).expect("実行済み通常技とDI被弾を提示");
    assert_eq!(card.id, "committed_button_vs_di");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::High);
    assert_eq!(card.evidence[0].frame, 990);
    assert_eq!(card.evidence[0].end_frame, Some(1120));
    assert!(card.evidence[0].label.contains("強K中に相手DI"));
    assert_invites_user_review(&card);

    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "原因が確定したDI被弾を守勢の暴れと重複分類しない"
    );
    assert!(
        detect_big_hits(&events, 1, &[card]).is_none(),
        "専用カードが同じ大被弾を所有する"
    );
}

#[test]
fn di_hit_without_confirmed_normal_execution_stays_in_tactic_stats_only() {
    let mut events = empty_events();
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 1.0,
        hp_after: 0.8,
        drop: 0.2,
        round_no: 1,
    });
    events.drive_impacts.push(DriveImpactEvent {
        side: 2,
        input_frame: 970,
        active_frame: Some(997),
        contact_frame: Some(1000),
        outcome: DriveImpactOutcome::Hit,
        damage: 0.2,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    assert!(
        detect_committed_button_vs_di(&events, 1, 0).is_none(),
        "通常技入力と実行証拠が無いDI反応一般はカードにしない"
    );
}
