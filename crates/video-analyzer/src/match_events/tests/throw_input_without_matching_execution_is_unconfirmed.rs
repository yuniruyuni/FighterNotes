use super::support::*;

#[test]
fn throw_input_without_matching_execution_is_unconfirmed() {
    use MeterState::*;
    let n = 180usize;
    let mut p1 = vec![Free; n];
    p1[100..104].fill(Startup);
    p1[124..130].fill(Active);
    let segment = InputSegment {
        start_frame: 100,
        end_frame: 104,
        dir: "N".to_string(),
        badges: vec![],
        auto: false,
        throw: true,
        evidence: Default::default(),
    };
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let events = super::actions::extract_throw_actions(
        &[p1, vec![Free; n]],
        &[vec![0; n], vec![0; n]],
        &[],
        &[],
        &[vec![segment], vec![]],
        &rounds,
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, ThrowOutcome::Unconfirmed);
    assert_ne!(events[0].confidence, EventConfidence::High);
}
