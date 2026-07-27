use super::support::*;

#[test]
fn test_mashing_ignores_projectile_block_context() {
    use crate::match_events::{ContactEvent, InputSegment, MeterState};
    let mut ev = empty_events();
    let n = 6000;
    ev.meter_state = [vec![MeterState::Free; n], vec![MeterState::Free; n]];
    // 大被弾 + 直前ボタン（被圧の材料は「弾ガード接触」のみ）
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 0.9,
        hp_after: 0.78,
        drop: 0.12,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1200,
        pre_freeze_frame: 1200,
        end_frame: 1220,
        hp_before: 0.78,
        hp_after: 0.66,
        drop: 0.12,
        round_no: 1,
    });
    let press = |start_frame| InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    ev.segments[0] = vec![press(990)];
    ev.contacts.push(ContactEvent {
        frame: 900,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: true,
        round_no: 1,
    });
    // 因果検証を通すため、被弾時に自分の技が動作中（発生を潰された）にする
    for k in 997..=1000 {
        ev.meter_state[0][k] = MeterState::Startup;
    }
    let report = build_report(&[], &ev, "p1", None);
    assert!(
        report.cards.iter().all(|c| c.id != "mashing"),
        "弾ガードは被圧の証拠にしない"
    );

    // 対照: 非弾（密着）ガードなら被圧として計上される
    ev.contacts[0].projectile = false;
    let report = build_report(&[], &ev, "p1", None);
    assert!(report.cards.iter().any(|c| c.id == "mashing"));
}
