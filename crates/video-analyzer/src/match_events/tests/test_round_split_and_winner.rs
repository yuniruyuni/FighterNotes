use super::support::*;

#[test]
fn test_round_split_and_winner() {
    let fs = synth_two_rounds();
    let ev = build_match_events(&fs, &[], &[], None, "p1");
    assert_eq!(
        ev.rounds.len(),
        2,
        "2 ラウンドに分割されるべき: {:?}",
        ev.rounds
    );
    assert_eq!(ev.rounds[0].winner, Some(1));
    assert_eq!(ev.rounds[1].winner, Some(2));
}
