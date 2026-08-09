use crate::test_support::*;

#[test]
fn minus_situation_keeps_observed_non_fastest_response_as_denominator() {
    let (ms, contacts, mut segs, rounds) = minus_press_fixture();
    segs[0] = vec![InputSegment {
        start_frame: 110,
        end_frame: 140,
        dir: "R".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);
    assert!(extracted.presses.is_empty());
    assert_eq!(extracted.situations.len(), 1);
    let situation = &extracted.situations[0];
    assert_eq!(situation.frame, 120);
    assert_eq!(situation.minus_frames, 5);
    assert_eq!(situation.fastest_action, None);
    assert_eq!(situation.outcome, None);
    assert_eq!(situation.confidence, EventConfidence::High);
}
