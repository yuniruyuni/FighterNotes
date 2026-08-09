use crate::test_support::*;

#[test]
fn minus_situation_links_confirmed_fastest_action_and_outcome() {
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    segs[0] = vec![minus_press(118)];
    contacts.push(ContactEvent {
        frame: 130,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 130,
        pre_freeze_frame: 130,
        end_frame: 145,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    }];

    let extracted = extract_minus_all(&ms, &contacts, &damage, &segs, &rounds);
    assert_eq!(extracted.presses.len(), 1);
    assert_eq!(extracted.situations.len(), 1);
    let situation = &extracted.situations[0];
    assert_eq!(situation.fastest_action, Some(DefensiveActionKind::Strike));
    assert_eq!(situation.action_frame, Some(120));
    assert_eq!(situation.outcome, Some(MinusPressOutcome::CounterHit));
    assert!((situation.drop - 0.12).abs() < 1e-6);
}
