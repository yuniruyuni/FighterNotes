use super::support::*;

#[test]
fn test_throw_loop_needs_streak() {
    let mut ev = empty_events();
    // 成功 2 連続までは確認場面として残すが、原因診断にはしない
    ev.throws = vec![
        ThrowEvent {
            thrower: 2,
            frame: 1000,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 1300,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 1600,
            connected: false,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 1900,
            connected: true,
            round_no: 1,
        },
    ];
    let report = detector_test_report(&ev, "p1");
    let card = report.cards.iter().find(|c| c.id == "throw_loop").unwrap();
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_invites_user_review(card);
    assert_eq!(card.evidence.len(), 3);

    // 3 連続 → カードあり
    ev.throws = vec![
        ThrowEvent {
            thrower: 2,
            frame: 1000,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 1300,
            connected: true,
            round_no: 1,
        },
        ThrowEvent {
            thrower: 2,
            frame: 1600,
            connected: true,
            round_no: 1,
        },
    ];
    let report = detector_test_report(&ev, "p1");
    let card = report.cards.iter().find(|c| c.id == "throw_loop").unwrap();
    assert_eq!(card.kind, AdviceKind::Diagnosis);
}
