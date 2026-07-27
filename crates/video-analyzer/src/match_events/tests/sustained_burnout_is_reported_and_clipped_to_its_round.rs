use super::support::*;

#[test]
fn sustained_burnout_is_reported_and_clipped_to_its_round() {
    let mut features = synth_two_rounds();
    for feature in &mut features[150..350] {
        feature.left_burnout = true;
    }

    let events = build_match_events(&features, &[], &[], None, "p1");
    let burnout = events
        .burnouts
        .iter()
        .find(|period| period.side == 1)
        .expect("P1 burnout period");

    assert_eq!(burnout.start_frame, 150);
    assert_eq!(burnout.end_frame, 349);
    assert_eq!(burnout.round_no, 1);
    assert_eq!(burnout.cause, BurnoutCause::Unknown);
    assert_eq!(burnout.confidence, EventConfidence::Medium);
}
