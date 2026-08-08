use match_event_layer::test_support::*;
use video_analyzer::advice;

#[test]
fn three_round_match_maps_winners_to_p2_report() {
    let features = synth_three_rounds_for_p2();
    let events = build_match_events(&features, &[], &[], None, "p2");
    assert_eq!(
        events
            .rounds
            .iter()
            .map(|round| (round.round_no, round.winner))
            .collect::<Vec<_>>(),
        [(1, Some(2)), (2, Some(1)), (3, Some(2))]
    );

    let report = advice::build_report(&features, &events, "p2", Some("KEN"));
    assert_eq!(
        report
            .round_summaries
            .iter()
            .map(|round| (
                round.round_no,
                round.won,
                round.detection_confidence.as_str()
            ))
            .collect::<Vec<_>>(),
        [
            (1, Some(true), "high"),
            (2, Some(false), "high"),
            (3, Some(true), "high"),
        ]
    );
}
