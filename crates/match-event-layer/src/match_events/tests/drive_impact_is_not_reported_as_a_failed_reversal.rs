use super::support::*;

#[test]
fn drive_impact_is_not_reported_as_a_failed_reversal() {
    use MeterState::*;
    let n = 200usize;
    let features: Vec<_> = (0..n).map(|frame| feat(frame as u32, 1.0, 1.0)).collect();
    let mut p1 = vec![Free; n];
    let mut p2 = vec![Free; n];
    p1[100..105].fill(Startup);
    p1[105..113].fill(Invincible);
    p1[113..121].fill(Active);
    p2[110..115].fill(Startup);
    p2[115..126].fill(Active);
    let meter_state = [p1, p2];
    let epochs = [vec![0; n], vec![0; n]];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 130,
        pre_freeze_frame: 130,
        end_frame: 145,
        hp_before: 1.0,
        hp_after: 0.85,
        drop: 0.15,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 0.85,
        p2_hp_end: 1.0,
    }];
    let di = |frame| InputSegment {
        start_frame: frame,
        end_frame: frame + 2,
        dir: "N".to_string(),
        badges: vec!["DI".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };

    let baseline = super::reversals::extract_reversals(super::reversals::ReversalInputs {
        features: &features,
        meter_state: &meter_state,
        meter_epoch: &epochs,
        contacts: &[],
        damage: &damage,
        segments: &[vec![], vec![di(110)]],
        rounds: &rounds,
        teleports: &[],
    });
    assert!(baseline.iter().any(|event| event.side == 1));

    let segments = [vec![di(100)], vec![di(110)]];
    let impacts = super::actions::extract_drive_impacts(
        &meter_state,
        &epochs,
        &[],
        &damage,
        &segments,
        &rounds,
    );
    assert!(impacts.iter().any(|impact| {
        impact.side == 1
            && impact.outcome == DriveImpactOutcome::Countered
            && impact.confidence == EventConfidence::High
    }));
    let reversals = super::reversals::extract_reversals(super::reversals::ReversalInputs {
        features: &features,
        meter_state: &meter_state,
        meter_epoch: &epochs,
        contacts: &[],
        damage: &damage,
        segments: &segments,
        rounds: &rounds,
        teleports: &[],
    });
    assert!(
        reversals.iter().all(|event| event.side != 1),
        "DIのアーマーを無敵技として二重帰属しない: {reversals:?}"
    );
}
