use crate::test_support::*;

#[test]
fn fragmented_invincible_run_is_one_reversal() {
    use MeterState::*;
    let n = 180usize;
    let features: Vec<_> = (0..n).map(|frame| feat(frame as u32, 1.0, 1.0)).collect();
    let mut p1 = vec![Free; n];
    let p2 = vec![Free; n];
    p1[100] = Invincible;
    p1[101] = ProjectileActive;
    p1[102] = Invincible;
    p1[103] = Stun;
    p1[104..107].fill(Invincible);
    p1[109..113].fill(Active);
    let meter_state = [p1, p2];
    let epochs = [vec![7; n], vec![7; n]];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 130,
        pre_freeze_frame: 130,
        end_frame: 145,
        hp_before: 1.0,
        hp_after: 0.76,
        drop: 0.24,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 0.76,
        p2_hp_end: 1.0,
    }];
    let dp = InputSegment {
        start_frame: 98,
        end_frame: 100,
        dir: "R".to_string(),
        badges: vec!["DP".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };

    let reversals = crate::reversals::extract_reversals(crate::reversals::ReversalInputs {
        features: &features,
        meter_state: &meter_state,
        meter_epoch: &epochs,
        contacts: &[],
        damage: &damage,
        segments: &[vec![dp], vec![]],
        rounds: &rounds,
        teleports: &[],
    });
    assert_eq!(
        reversals.len(),
        1,
        "分断された同じ無敵技を重複させない: {reversals:?}"
    );
    assert_eq!((reversals[0].frame, reversals[0].drop), (100, 0.24));
    assert_eq!(reversals[0].confidence, EventConfidence::High);
}
