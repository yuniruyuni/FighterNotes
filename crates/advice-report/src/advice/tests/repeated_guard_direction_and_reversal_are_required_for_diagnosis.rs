use super::support::*;

#[test]
fn repeated_guard_direction_and_reversal_are_required_for_diagnosis() {
    use crate::match_events::{GuardBreakEvent, ReversalEvent};

    let mut guard = empty_events();
    let guard_event = |frame| GuardBreakEvent {
        side: 1,
        frame,
        drop: 0.1,
        guard_dir: "DR".to_string(),
        broke_to: "R".to_string(),
        round_no: 1,
    };
    guard.guard_breaks.push(guard_event(1000));
    assert_eq!(
        detect_guard_break(&guard, 1).unwrap().kind,
        AdviceKind::Observation
    );
    guard.guard_breaks.push(guard_event(2000));
    assert_eq!(
        detect_guard_break(&guard, 1).unwrap().kind,
        AdviceKind::Diagnosis
    );

    let mut reversal = empty_events();
    let reversal_event = |frame| ReversalEvent {
        side: 1,
        frame,
        drop: 0.2,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    };
    reversal.reversals.push(reversal_event(1000));
    assert_eq!(
        detect_reversal_punished(&reversal, 1).unwrap().kind,
        AdviceKind::Observation
    );
    reversal.reversals.push(reversal_event(2000));
    assert_eq!(
        detect_reversal_punished(&reversal, 1).unwrap().kind,
        AdviceKind::Diagnosis
    );
}
