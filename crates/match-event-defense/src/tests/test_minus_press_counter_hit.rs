use crate::test_support::*;

#[test]
fn test_minus_press_counter_hit() {
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    segs[0] = vec![minus_press(118)]; // 硬直中の仕込み押し
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
    let ev = extract_minus(&ms, &contacts, &damage, &segs, &rounds);
    assert_eq!(ev.len(), 1, "{ev:?}");
    assert_eq!(ev[0].minus_frames, 5);
    assert_eq!(ev[0].outcome, MinusPressOutcome::CounterHit);
    assert!((ev[0].drop - 0.12).abs() < 1e-6);
    assert_eq!(ev[0].pressed, "弱");
}
