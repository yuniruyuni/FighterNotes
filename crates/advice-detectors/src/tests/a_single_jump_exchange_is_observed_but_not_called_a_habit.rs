use super::support::*;

#[test]
fn a_single_jump_exchange_is_observed_but_not_called_a_habit() {
    let jump = |side, frame, outcome| JumpEvent {
        side,
        frame,
        outcome,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: true,
        air_end: frame + 47,
        round_no: 1,
    };

    let mut anti_air = empty_events();
    anti_air.jumps.push(jump(2, 1000, JumpOutcome::LandedHit));
    assert_eq!(
        detect_anti_air(&anti_air, 1, 2).unwrap().kind,
        AdviceKind::Observation
    );
    anti_air.jumps.push(jump(2, 2000, JumpOutcome::LandedHit));
    assert_eq!(
        detect_anti_air(&anti_air, 1, 2).unwrap().kind,
        AdviceKind::Diagnosis
    );

    for frame in [3000, 4000, 5000] {
        anti_air.jumps.push(jump(2, frame, JumpOutcome::GotHit));
    }
    assert_eq!(
        detect_anti_air(&anti_air, 1, 2).unwrap().kind,
        AdviceKind::Observation,
        "飛びの攻防が2失敗/5成立なら被弾は見せるが高い失敗率とは断定しない"
    );

    let mut own_jump = empty_events();
    own_jump.jumps.push(jump(1, 1000, JumpOutcome::GotHit));
    assert_eq!(
        detect_own_jumps(&own_jump, 1).unwrap().kind,
        AdviceKind::Observation
    );
    own_jump.jumps.push(jump(1, 2000, JumpOutcome::GotHit));
    assert_eq!(
        detect_own_jumps(&own_jump, 1).unwrap().kind,
        AdviceKind::Diagnosis
    );
}
