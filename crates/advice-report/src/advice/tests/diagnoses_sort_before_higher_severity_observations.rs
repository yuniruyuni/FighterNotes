use super::support::*;

#[test]
fn diagnoses_sort_before_higher_severity_observations() {
    let mut ev = empty_events();
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1050,
        hp_before: 1.0,
        hp_after: 0.5,
        drop: 0.5,
        round_no: 1,
    });
    ev.throws = vec![
        ThrowEvent {
            thrower: 2,
            frame: 100,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 200,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 300,
            connected: true,
            round_no: 1,
        },
    ];

    let report = detector_test_report(&ev, "p1");
    assert_eq!(report.ruleset_version, 16);
    assert_eq!(report.cards[0].id, "throw_loop");
    assert_eq!(report.cards[0].kind, AdviceKind::Diagnosis);
    assert_eq!(report.cards[1].id, "big_hits");
    assert!(report
        .summary
        .ends_with("優先改善: 投げを連続して受けている"));
}
