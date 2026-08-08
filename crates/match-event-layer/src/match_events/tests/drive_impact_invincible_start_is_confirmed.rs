use super::support::*;

#[test]
fn drive_impact_invincible_to_active_is_confirmed() {
    use MeterState::*;

    let n = 160usize;
    let mut p1 = vec![Free; n];
    p1[50..58].fill(Invincible);
    p1[58..66].fill(Active);
    let meter_state = [p1, vec![Free; n]];
    let epochs = [vec![0; n], vec![0; n]];
    let contacts = vec![ContactEvent {
        frame: 60,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let di = InputSegment {
        start_frame: 50,
        end_frame: 52,
        dir: "N".to_string(),
        badges: vec!["DI".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };

    let impacts = super::actions::extract_drive_impacts(
        &meter_state,
        &epochs,
        &contacts,
        &[],
        &[vec![di], vec![]],
        &rounds,
    );

    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].active_frame, Some(58));
    assert_eq!(impacts[0].contact_frame, Some(60));
    assert_eq!(impacts[0].outcome, DriveImpactOutcome::Blocked);
    assert_eq!(impacts[0].confidence, EventConfidence::High);
}

#[test]
fn drive_impact_parry_armor_survives_video_frame_stretch() {
    use MeterState::*;

    let n = 180usize;
    let mut p1 = vec![Free; n];
    p1[50..105].fill(Parry);
    p1[105..113].fill(Active);
    let meter_state = [p1, vec![Free; n]];
    let epochs = [vec![0; n], vec![0; n]];
    let contacts = vec![ContactEvent {
        frame: 108,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let di = InputSegment {
        start_frame: 50,
        end_frame: 52,
        dir: "N".to_string(),
        badges: vec!["強P".to_string(), "強K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };

    let impacts = super::actions::extract_drive_impacts(
        &meter_state,
        &epochs,
        &contacts,
        &[],
        &[vec![di], vec![]],
        &rounds,
    );

    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].active_frame, Some(105));
    assert_eq!(impacts[0].contact_frame, Some(108));
    assert_eq!(impacts[0].outcome, DriveImpactOutcome::Blocked);
    assert_eq!(impacts[0].confidence, EventConfidence::High);
}
