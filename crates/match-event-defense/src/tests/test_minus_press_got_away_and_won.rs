use crate::test_support::*;

#[test]
fn test_minus_press_got_away_and_won() {
    // 被弾なし → GotAway
    let (ms, contacts, mut segs, rounds) = minus_press_fixture();
    segs[0] = vec![minus_press(118)];
    let ev = extract_minus(&ms, &contacts, &[], &segs, &rounds);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].outcome, MinusPressOutcome::GotAway);

    // 押下後に自分が attacker の接触 → Won
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    segs[0] = vec![minus_press(118)];
    contacts.push(ContactEvent {
        frame: 128,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    let ev = extract_minus(&ms, &contacts, &[], &segs, &rounds);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].outcome, MinusPressOutcome::Won);
}
