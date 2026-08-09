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
        advice_card("mashing", Some(300)),
    ];

    apply_advice_contexts(&mut breakdown, &cards);

    assert_eq!(breakdown.events[0].contexts, [DamageContext::Mashing]);
    assert!(breakdown.events[1].contexts.is_empty());
}
