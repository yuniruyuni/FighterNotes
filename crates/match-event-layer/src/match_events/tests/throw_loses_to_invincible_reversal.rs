use super::support::*;

#[test]
fn executed_throw_interrupted_by_invincible_reversal_is_not_a_whiff() {
    use MeterState::*;
    let n = 220usize;
    let mut p1 = vec![Free; n];
    let mut p2 = vec![Free; n];
    p1[100..104].fill(Startup);
    p1[104..110].fill(Active);
    p2[103..109].fill(Invincible);
    p2[109..120].fill(Active);
    p1[109..120].fill(Stun);
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 109,
        pre_freeze_frame: 109,
        end_frame: 145,
        hp_before: 1.0,
        hp_after: 0.87,
        drop: 0.13,
        round_no: 1,
    }];
    let segment = InputSegment {
        start_frame: 100,
        end_frame: 106,
        dir: "R".to_string(),
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
        p1_hp_end: 0.87,
        p2_hp_end: 1.0,
    }];

    let events = super::actions::extract_throw_actions(
        &[p1, p2],
        &[vec![0; n], vec![0; n]],
        &[],
        &damage,
        &[vec![segment], vec![]],
        &rounds,
    );

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, ThrowOutcome::InterruptedByInvincible);
    assert_eq!(events[0].confidence, EventConfidence::High);
}
