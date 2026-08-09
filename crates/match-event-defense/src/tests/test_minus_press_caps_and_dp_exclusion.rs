use crate::test_support::*;

#[test]
fn test_minus_press_caps_and_dp_exclusion() {
    use MeterState::*;
    // 不利幅が上限超（ダウン・被コンボ由来）→ 対象外:
    // 自分の Stun を f100..160 に延長（不利 45F）
    let (mut ms, contacts, mut segs, rounds) = minus_press_fixture();
    for s in ms[0].iter_mut().take(160).skip(100) {
        *s = Stun;
    }
    segs[0] = vec![minus_press(150)];
    assert!(
        extract_minus(&ms, &contacts, &[], &segs, &rounds).is_empty(),
        "ノックダウン級の長い硬直はガード後の不利として扱わない"
    );

    // DP バッジの押下 → 除外（硬直中の仕込みだと技が出ず Invincible が
    // メーターに現れないため、バッジで判定する）
    let (ms, contacts, mut segs, rounds) = minus_press_fixture();
    let mut dp = minus_press(118);
    dp.badges = vec!["DP".to_string()];
    segs[0] = vec![dp];
    assert!(extract_minus(&ms, &contacts, &[], &segs, &rounds).is_empty());
}
