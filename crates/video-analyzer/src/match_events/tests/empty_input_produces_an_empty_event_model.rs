use super::support::*;

#[test]
fn empty_input_produces_an_empty_event_model() {
    let events = build_match_events(&[], &[], &[], None, "p1");

    assert!(events.rounds.is_empty());
    assert!(events.damage.is_empty());
    assert!(events.jumps.is_empty());
    assert!(events.burnouts.is_empty());
    assert!(events.segments.iter().all(Vec::is_empty));
    assert!(events.hp.iter().all(Vec::is_empty));
}
