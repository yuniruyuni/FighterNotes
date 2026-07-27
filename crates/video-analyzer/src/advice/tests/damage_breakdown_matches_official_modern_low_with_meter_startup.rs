use super::support::*;

#[test]
fn damage_breakdown_matches_official_modern_low_with_meter_startup() {
    use crate::match_events::{ContactEvent, InputSegment, MeterState};

    let features: Vec<_> = (0..30u32)
        .map(|frame_index| FrameFeatures {
            frame_index,
            fps: 60.0,
            own_hp: 1.0,
            opponent_hp: 1.0,
            is_match_screen: true,
            own_meter_state: None,
            opponent_meter_state: None,
            left_hp_score: 1.0,
            right_hp_score: 1.0,
            left_drive_ratio: 1.0,
            right_drive_ratio: 1.0,
            left_burnout: false,
            right_burnout: false,
            left_drive_uncertain: false,
            right_drive_uncertain: false,
            left_hp_raw: 1.0,
            right_hp_raw: 1.0,
            left_hp_raw_quality: 0.0,
            right_hp_raw_quality: 0.0,
        })
        .collect();
    let mut events = empty_events();
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 14,
        pre_freeze_frame: 14,
        end_frame: 20,
        hp_before: 1.0,
        hp_after: 0.98,
        drop: 0.02,
        round_no: 1,
    });
    events.contacts.push(ContactEvent {
        frame: 14,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    events.segments[1].push(InputSegment {
        start_frame: 10,
        end_frame: 10,
        dir: "D".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    events.meter_state[1] = vec![MeterState::Free; 30];
    events.meter_state[1][10..14].fill(MeterState::Startup);
    events.meter_state[1][14] = MeterState::Active;
    events.meter_game_frame[1] = (0..30).map(i64::from).collect();

    let breakdown =
        super::damage_origins::build_damage_breakdown(&features, &events, 1, Some("INGRID"));
    let event = &breakdown.events[0];
    assert_eq!(event.origin, DamageOrigin::Strike);
    assert_eq!(event.strike_kind, Some(crate::frame_data::StrikeKind::Low));
    assert_eq!(event.strike_kind_confidence, Some(EventConfidence::High));
}
