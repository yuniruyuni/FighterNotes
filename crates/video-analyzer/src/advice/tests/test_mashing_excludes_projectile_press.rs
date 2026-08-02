use super::support::*;

#[test]
fn test_mashing_excludes_projectile_press() {
    use crate::match_events::{InputSegment, MeterState};
    let mut ev = empty_events();
    // 被圧の材料（直近被弾）+ 大被弾 + 直前ボタン
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
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
    let projectile_press = |start_frame| InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["SP".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    ev.segments[0] = vec![projectile_press(990), projectile_press(1190)];
    // 押下直後に自分側メーターへ弾判定が出ている = 弾を撃った
    let n = 6000;
    let mut own_state = vec![MeterState::Free; n];
    for s in own_state.iter_mut().take(1000).skip(995) {
        *s = MeterState::ProjectileActive;
    }
    for s in own_state.iter_mut().take(1200).skip(1195) {
        *s = MeterState::ProjectileActive;
    }
    ev.meter_state = [own_state, vec![MeterState::Free; n]];
    let report = detector_test_report(&ev, "p1");
    assert!(
        report.cards.iter().all(|c| c.id != "mashing"),
        "弾を撃った押しは暴れにしない"
    );

    // 対照: メーター無し（弾の証拠なし）なら計上される
    ev.meter_state = [vec![], vec![]];
    let report = detector_test_report(&ev, "p1");
    assert!(report.cards.iter().any(|c| c.id == "mashing"));
}
