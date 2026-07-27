use super::support::*;

#[test]
fn test_big_hits_card() {
    let mut ev = empty_events();
    // 閾値未満(0.17)は対象外 / 閾値以上(0.30)は列挙 / 相手側の被弾は対象外
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 500,
        pre_freeze_frame: 500,
        end_frame: 520,
        hp_before: 1.0,
        hp_after: 0.83,
        drop: 0.17,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1050,
        hp_before: 0.83,
        hp_after: 0.53,
        drop: 0.30,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
        victim: 2,
        start_frame: 1500,
        pre_freeze_frame: 1500,
        end_frame: 1550,
        hp_before: 1.0,
        hp_after: 0.5,
        drop: 0.5,
        round_no: 1,
    });
    let report = build_report(&[], &ev, "p1", None);
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "big_hits")
        .expect("大ダメージカードが出るべき");
    assert_eq!(card.evidence.len(), 1, "{:?}", card.evidence);
    assert_eq!(card.evidence[0].frame, 1000);
    assert!((card.severity - 0.30).abs() < 1e-6);
    assert!(card.evidence[0].label.contains("-30%"));
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::High);
    assert!(report.summary.contains("原因を断定できる改善指摘はなく"));
}
