use super::*;

#[test]
fn projectile_trajectory_confidence_propagates_to_compound_threat() {
    let mut events = empty_events();
    events.projectiles.push(ProjectileThreat {
        owner: 1,
        observed_start_frame: 100,
        observed_end_frame: 110,
        threat_end_frame: 120,
        contact_frame: Some(120),
        round_no: 1,
        confidence: 0.75,
    });
    events.compound_threats.push(CompoundThreat {
        attacker: 1,
        defender: 2,
        projectile_start_frame: 100,
        teleport_frame: 105,
        followup_attack_frame: 115,
        followup_contact_frame: Some(120),
        projectile_response: None,
        followup_response: None,
        outcome: ThreatOutcome::Defended,
        damage: 0.0,
        round_no: 1,
        confidence: 0.8,
    });
    let observations = [SpatialObservation {
        frame_index: 108,
        p1: None,
        p2: None,
        screen_distance: None,
        distance_band: None,
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![ProjectileCandidate {
            track_id: 1,
            center: SpatialPoint::new(0.5, 0.5),
            bounds: SpatialRect::new(0.48, 0.48, 0.52, 0.52),
            velocity_x: Some(0.02),
            motion: HorizontalMotion::Right,
            trajectory_confirmed: true,
            confidence: 0.8,
        }],
        motion_regions: vec![],
        contact: None,
        camera: None,
    }];

    refine_match_events_with_spatial(&mut events, &observations, &AnalysisContext::default());

    assert_eq!(events.projectiles[0].confidence, 0.95);
    assert_eq!(events.compound_threats[0].confidence, 0.95);
}
