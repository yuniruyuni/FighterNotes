use super::support::*;

#[test]
fn backward_jump_hit_does_not_emit_approach_advice() {
    let mut ev = empty_events();
    ev.jumps.push(JumpEvent {
        side: 1,
        frame: 100,
        outcome: JumpOutcome::GotHit,
        input_dir: "UL".to_string(),
        direction: JumpDirection::Backward,
        contact_frame: Some(120),
        takeoff_confirmed: true,
        air_end: 147,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 120,
        end_frame: 130,
        pre_freeze_frame: 120,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    });

    assert!(detect_own_jumps(&ev, 1).is_none());

    ev.jumps[0].direction = JumpDirection::Forward;
    ev.jumps[0].outcome = JumpOutcome::UnverifiedHit;
    ev.jumps[0].takeoff_confirmed = false;
    assert!(detect_own_jumps(&ev, 1).is_none());
}
