use super::super::{apply_advice_contexts, build_damage_breakdown};
use super::support::{advice_card, damage, empty_events};
use crate::DamageContext;

#[test]
fn mashing_context_uses_nearest_end_frame_and_deduplicates() {
    let mut events = empty_events();
    events.damage = vec![damage(100, 1, 0.1), damage(200, 1, 0.1)];
    let mut breakdown = build_damage_breakdown(&[], &events, 1, None);
    let cards = [
        advice_card("mashing", Some(115)),
        advice_card("mashing", Some(115)),
        advice_card("other", Some(212)),
        advice_card("mashing", None),
        advice_card("mashing", Some(212)),
        advice_card("mashing", Some(300)),
    ];

    apply_advice_contexts(&mut breakdown, &cards);

    assert_eq!(breakdown.events[0].contexts, [DamageContext::Mashing]);
    assert_eq!(breakdown.events[1].contexts, [DamageContext::Mashing]);
}

#[test]
fn mashing_context_includes_the_exact_matching_edge() {
    let mut events = empty_events();
    events.damage = vec![damage(100, 1, 0.1)];
    let mut breakdown = build_damage_breakdown(&[], &events, 1, None);

    apply_advice_contexts(&mut breakdown, &[advice_card("mashing", Some(117))]);

    assert_eq!(breakdown.events[0].contexts, [DamageContext::Mashing]);
}

#[test]
fn mashing_context_ignores_evidence_outside_the_matching_window() {
    let mut events = empty_events();
    events.damage = vec![damage(100, 1, 0.1)];
    let mut breakdown = build_damage_breakdown(&[], &events, 1, None);

    apply_advice_contexts(&mut breakdown, &[advice_card("mashing", Some(300))]);

    assert!(breakdown.events[0].contexts.is_empty());
}

#[test]
fn advice_contexts_are_a_noop_without_damage_events() {
    let events = empty_events();
    let mut breakdown = build_damage_breakdown(&[], &events, 1, None);

    apply_advice_contexts(&mut breakdown, &[advice_card("mashing", Some(100))]);

    assert!(breakdown.events.is_empty());
}
