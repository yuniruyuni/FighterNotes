use super::support::*;

#[test]
fn test_minus_press_threshold_and_exclusions() {
    use MeterState::*;
    // 不利 2Fも最速Startupが確認できれば記録する。
    let (mut ms, contacts, mut segs, rounds) = minus_press_fixture();
    for s in ms[1].iter_mut().take(118).skip(105) {
        *s = MotionRecovery;
    }
    segs[0] = vec![minus_press(118)];
    let events = extract_minus(&ms, &contacts, &[], &segs, &rounds);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].minus_frames, 2);

    // 投げ入力も独立した最速防御行動として記録する。
    let (ms, contacts, mut segs, rounds) = minus_press_fixture();
    let mut t = minus_press(118);
    t.badges.clear();
    t.throw = true;
    segs[0] = vec![t];
    let events = extract_minus(&ms, &contacts, &[], &segs, &rounds);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action_kind, DefensiveActionKind::Throw);

    // 五分は「不利からの最速行動」ではない。
    let (mut ms, contacts, mut segs, rounds) = minus_press_fixture();
    for state in ms[1].iter_mut().take(120).skip(105) {
        *state = MotionRecovery;
    }
    segs[0] = vec![minus_press(118)];
    assert!(extract_minus(&ms, &contacts, &[], &segs, &rounds).is_empty());

    // 無敵技（押下直後に自分が Invincible）→ 除外
    let (mut ms, contacts, mut segs, rounds) = minus_press_fixture();
    for s in ms[0].iter_mut().take(130).skip(121) {
        *s = Invincible;
    }
    segs[0] = vec![minus_press(118)];
    assert!(extract_minus(&ms, &contacts, &[], &segs, &rounds).is_empty());

    // 弾ガード接触 → 対象外（遠距離の弾ガード後の反撃は安全な行動）
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    contacts[0].projectile = true;
    segs[0] = vec![minus_press(118)];
    assert!(extract_minus(&ms, &contacts, &[], &segs, &rounds).is_empty());

    // 同一押下セグメントは 1 回だけ（多段ガードの重複排除）
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    contacts.push(ContactEvent {
        frame: 102,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    });
    segs[0] = vec![minus_press(118)];
    let ev = extract_minus(&ms, &contacts, &[], &segs, &rounds);
    assert_eq!(ev.len(), 1, "{ev:?}");
}
