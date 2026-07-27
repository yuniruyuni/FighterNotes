use super::support::*;

#[test]
fn mashing_requires_direct_input_and_reliable_linked_startup() {
    use crate::match_events::{InputEvidence, MeterState};

    let mut ev = basic_mashing_events();
    ev.segments[0][0].evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 6,
    };
    let card = detect_mashing(&[], &ev, 1, 0).expect("直接観測できた2件目だけを残す");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].frame, 1190);

    ev.segments[0][0].evidence = Default::default();
    let n = 6000;
    let mut own_state = vec![MeterState::Free; n];
    for state in own_state.iter_mut().take(1001).skip(997) {
        *state = MeterState::Active;
    }
    for state in own_state.iter_mut().take(1201).skip(1197) {
        *state = MeterState::Active;
    }
    ev.meter_state = [own_state, vec![MeterState::Free; n]];
    ev.meter_confidence = [vec![1.0; n], vec![1.0; n]];
    assert!(
        detect_mashing(&[], &ev, 1, 0).is_none(),
        "被弾時に技動作中でも、入力直後の Startup が無ければ帰属しない"
    );

    ev.meter_state[0][997] = MeterState::Startup;
    ev.meter_state[0][1197] = MeterState::Startup;
    ev.meter_confidence[0][997] = 0.3;
    ev.meter_confidence[0][1197] = 0.3;
    assert!(
        detect_mashing(&[], &ev, 1, 0).is_none(),
        "低確度の Startup は因果根拠にしない"
    );

    ev.meter_confidence[0][997] = 1.0;
    ev.meter_confidence[0][1197] = 1.0;
    let card = detect_mashing(&[], &ev, 1, 0).expect("入力と技発生を結べる");
    assert_eq!(card.confidence, EventConfidence::High);
    assert_eq!(card.evidence[0].frame, 990);
    assert_eq!(card.evidence[0].end_frame, Some(1020));
}
