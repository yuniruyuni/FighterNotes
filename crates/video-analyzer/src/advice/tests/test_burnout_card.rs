use super::support::*;

#[test]
fn test_burnout_card() {
    let mut ev = empty_events();
    ev.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 2000,
        end_frame: 2900,
        hp_lost: 0.2,
        hp_dealt: 0.08,
        cause: crate::match_events::BurnoutCause::ForcedByGuard,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    let report = detector_test_report(&ev, "p1");
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "burnout")
        .expect("バーンアウトカード");
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.kind, AdviceKind::Statistic);
    assert_eq!(card.confidence, EventConfidence::High);
}
