use super::super::*;
use super::support::empty_events;
use crate::match_events::DamageEvent;
use crate::BIG_HIT_LIST;

#[test]
fn big_hit_threshold_is_inclusive() {
    let mut events = empty_events();
    events.damage = vec![
        DamageEvent {
            victim: 1,
            start_frame: 100,
            pre_freeze_frame: 95,
            end_frame: 130,
            hp_before: 1.0,
            hp_after: 1.0 - BIG_HIT_LIST,
            drop: BIG_HIT_LIST,
            round_no: 1,
        },
        DamageEvent {
            victim: 1,
            start_frame: 200,
            pre_freeze_frame: 195,
            end_frame: 230,
            hp_before: 1.0 - BIG_HIT_LIST,
            hp_after: 0.65,
            drop: BIG_HIT_LIST - 0.001,
            round_no: 1,
        },
    ];

    let card = detect_big_hits(&events, 1, &[]).expect("threshold hit should be listed");

    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].frame, 95);
    assert_eq!(card.evidence[0].end_frame, Some(130));
    assert!((card.severity - BIG_HIT_LIST).abs() < f32::EPSILON);
}
