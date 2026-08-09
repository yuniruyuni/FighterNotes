use super::super::build_damage_breakdown;
use super::support::{contact, damage, empty_events, features};
use crate::frame_data::StrikeKind;
use crate::match_events::{EventConfidence, InputEvidence, InputSegment, MeterState};

fn input_segment(dir: &str, badge: &str, auto: bool) -> InputSegment {
    InputSegment {
        start_frame: 10,
        end_frame: 10,
        dir: dir.to_string(),
        badges: vec![badge.to_string()],
        auto,
        throw: false,
        evidence: InputEvidence::default(),
    }
}

#[test]
fn strike_kind_without_meter_startup_has_medium_confidence() {
    let mut events = empty_events();
    events.damage.push(damage(14, 1, 0.02));
    events.contacts.push(contact(14, false));
    events.segments[1].push(input_segment("N", "弱", true));

    let breakdown = build_damage_breakdown(&[], &events, 1, Some("INGRID"));
    let event = &breakdown.events[0];

    assert_eq!(event.strike_kind, Some(StrikeKind::High));
    assert_eq!(event.strike_kind_confidence, Some(EventConfidence::Medium));
}

#[test]
fn strike_kind_requires_direct_input_observation() {
    let mut events = empty_events();
    events.damage.push(damage(14, 1, 0.02));
    events.contacts.push(contact(14, false));
    let mut segment = input_segment("N", "弱", true);
    segment.evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 1,
    };
    events.segments[1].push(segment);

    let breakdown = build_damage_breakdown(&[], &events, 1, Some("INGRID"));

    assert_eq!(breakdown.events[0].strike_kind, None);
    assert_eq!(breakdown.events[0].strike_kind_confidence, None);
}

#[test]
fn equally_near_inputs_with_different_kinds_remain_unknown() {
    let frames = features(30);
    let mut events = empty_events();
    events.damage.push(damage(14, 1, 0.02));
    events.contacts.push(contact(14, false));
    events.segments[1] = vec![
        input_segment("D", "弱", false),
        input_segment("N", "弱", true),
    ];
    events.meter_state[1] = vec![MeterState::Free; 30];
    events.meter_state[1][10..14].fill(MeterState::Startup);
    events.meter_state[1][14] = MeterState::Active;
    events.meter_game_frame[1] = (0..30).map(i64::from).collect();

    let breakdown = build_damage_breakdown(&frames, &events, 1, Some("INGRID"));

    assert_eq!(breakdown.events[0].strike_kind, None);
}

#[test]
fn interrupted_meter_startup_downgrades_input_match_to_medium() {
    let frames = features(30);
    let mut events = empty_events();
    events.damage.push(damage(14, 1, 0.02));
    events.contacts.push(contact(14, false));
    events.segments[1].push(input_segment("N", "弱", true));
    events.meter_state[1] = vec![MeterState::Free; 30];
    events.meter_state[1][10] = MeterState::Startup;
    events.meter_state[1][11] = MeterState::Startup;
    events.meter_state[1][12] = MeterState::Free;
    events.meter_state[1][14] = MeterState::Active;
    events.meter_game_frame[1] = (0..30).map(i64::from).collect();

    let breakdown = build_damage_breakdown(&frames, &events, 1, Some("INGRID"));

    assert_eq!(breakdown.events[0].strike_kind, Some(StrikeKind::High));
    assert_eq!(
        breakdown.events[0].strike_kind_confidence,
        Some(EventConfidence::Medium)
    );
}
