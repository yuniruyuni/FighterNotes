use super::super::{
    approximately_same_drop, contact_matches, frame_index, offer, segment_distance, starts_in,
    threat_confidence,
};
use super::support::{damage, features};
use crate::advice::DamageOrigin;
use crate::match_events::{EventConfidence, InputSegment};

#[test]
fn threat_confidence_uses_inclusive_medium_and_high_thresholds() {
    assert_eq!(threat_confidence(0.649), None);
    assert_eq!(threat_confidence(0.65), Some(EventConfidence::Medium));
    assert_eq!(threat_confidence(0.849), Some(EventConfidence::Medium));
    assert_eq!(threat_confidence(0.85), Some(EventConfidence::High));
}

#[test]
fn candidate_offer_orders_by_priority_confidence_then_distance() {
    let damage = damage(100, 1, 0.1);
    let mut best = None;
    offer(
        &mut best,
        DamageOrigin::Strike,
        EventConfidence::Low,
        100,
        100,
        &damage,
    );
    assert!(best.is_none(), "low-confidence candidates are ignored");

    offer(
        &mut best,
        DamageOrigin::Strike,
        EventConfidence::High,
        10,
        100,
        &damage,
    );
    offer(
        &mut best,
        DamageOrigin::Throw,
        EventConfidence::Medium,
        20,
        80,
        &damage,
    );
    assert_eq!(best.unwrap().origin, DamageOrigin::Throw);

    offer(
        &mut best,
        DamageOrigin::Projectile,
        EventConfidence::High,
        20,
        70,
        &damage,
    );
    assert_eq!(best.unwrap().origin, DamageOrigin::Projectile);

    offer(
        &mut best,
        DamageOrigin::Teleport,
        EventConfidence::High,
        20,
        95,
        &damage,
    );
    assert_eq!(best.unwrap().origin, DamageOrigin::Teleport);
}

#[test]
fn matching_windows_are_inclusive_at_both_edges() {
    let damage = damage(100, 1, 0.1);
    assert!(contact_matches(&damage, 75));
    assert!(!contact_matches(&damage, 74));
    assert!(contact_matches(&damage, 105));
    assert!(!contact_matches(&damage, 106));
    assert!(starts_in(&damage, 100, 100));
    assert!(approximately_same_drop(0.1, 0.104));
    assert!(!approximately_same_drop(0.1, 0.106));
}

#[test]
fn frame_and_segment_distance_helpers_cover_inside_and_missing_values() {
    let features = features(5);
    assert_eq!(frame_index(&features, 3), Some(3));
    assert_eq!(frame_index(&features, 8), None);

    let segment = InputSegment {
        start_frame: 10,
        end_frame: 15,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    assert_eq!(segment_distance(&segment, 7), 3);
    assert_eq!(segment_distance(&segment, 12), 0);
    assert_eq!(segment_distance(&segment, 20), 5);
}
