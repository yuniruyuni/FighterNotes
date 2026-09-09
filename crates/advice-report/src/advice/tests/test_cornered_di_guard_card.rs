use super::support::*;
use crate::match_events::{CornerSpan, DriveImpactEvent, DriveImpactOutcome};

/// 端でのDIガードは、イベントと corner span の交差からカードになって
/// レポートへ届く。結線を忘れても例外は出ず、カードが黙って消えるだけ
/// なので、ここで固定する。
#[test]
fn test_cornered_di_guard_card() {
    let mut ev = empty_events();
    ev.corner_spans.push(CornerSpan {
        side: 1,
        start_frame: 950,
        end_frame: 1000,
    });
    ev.drive_impacts.push(DriveImpactEvent {
        side: 2,
        input_frame: 974,
        active_frame: Some(998),
        contact_frame: Some(1000),
        outcome: DriveImpactOutcome::Blocked,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    let report = detector_test_report(&ev, "p1");
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "cornered_di_guard")
        .expect("端DIガードカード");
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::High);
}
