use super::support::*;

#[test]
fn test_mashing_excludes_beaten_by_invincible() {
    use crate::match_events::{InputSegment, MeterState};
    let mut ev = empty_events();
    let n = 6000;
    // 被圧の材料 + 大被弾 + 直前ボタン
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
    ev.segments[0] = vec![press(990), press(1190)];
    // 被弾直前に相手が無敵（弾抜け技）
    let mut opp_state = vec![MeterState::Free; n];
    for s in opp_state.iter_mut().take(998).skip(985) {
        *s = MeterState::Invincible;
    }
    ev.meter_state = [vec![MeterState::Free; n], opp_state];
    let report = build_report(&[], &ev, "p1", None);
    assert!(
        report.cards.iter().all(|c| c.id != "mashing"),
        "弾抜け（無敵技）に狩られた場面は暴れにしない"
    );
}
