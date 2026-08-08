use super::support::*;

#[test]
fn test_anti_air_card() {
    let mut ev = empty_events();
    // 相手(P2)のジャンプ 5 回: 3 通された, 1 対空, 1 なにもなし
    for (k, out) in [
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
        JumpOutcome::GotHit,
        JumpOutcome::Neutral,
    ]
    .iter()
    .enumerate()
    {
        ev.jumps.push(JumpEvent {
            side: 2,
            frame: 1000 + 500 * k as u32,
            outcome: *out,
            input_dir: "UR".to_string(),
            direction: JumpDirection::Forward,
            contact_frame: None,
            takeoff_confirmed: true,
            air_end: (1000 + 500 * k as u32) + 47,
            round_no: 1,
        });
    }
    // 通されたジャンプに対応する被弾
    for k in 0..3u32 {
        ev.damage.push(DamageEvent {
            victim: 1,
            start_frame: 1000 + 500 * k + 20,
            pre_freeze_frame: 1000 + 500 * k + 20,
            end_frame: 1000 + 500 * k + 40,
            hp_before: 1.0,
            hp_after: 0.9,
            drop: 0.1,
            round_no: 1,
        });
    }
    let report = detector_test_report(&ev, "p1");
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "anti_air")
        .expect("対空カードが出るべき");
    assert_eq!(card.evidence.len(), 3);
    assert!(card.severity > 0.3);
}
