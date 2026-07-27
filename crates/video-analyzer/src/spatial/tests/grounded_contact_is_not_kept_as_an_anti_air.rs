use super::*;

#[test]
fn grounded_contact_is_not_kept_as_an_anti_air() {
    let actor = |x, ground_anchor| ActorObservation {
        anchor: SpatialPoint::new(x, if ground_anchor { 0.9 } else { 0.65 }),
        bounds: SpatialRect::new(x - 0.03, 0.4, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor,
        discontinuity: false,
    };
    let observation = |frame_index, p2_grounded| SpatialObservation {
        frame_index,
        p1: Some(actor(0.35, true)),
        p2: Some(actor(0.65, p2_grounded)),
        screen_distance: Some(0.3),
        distance_band: Some(DistanceBand::Mid),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
    };
    let context = AnalysisContext::from_characters("p2", Some("LUKE"), Some("CHUN_LI"));

    let mut grounded = empty_events();
    grounded
        .jumps
        .push(jump(100, JumpOutcome::UnverifiedHit, "UR"));
    let grounded_observations: Vec<_> = (100..=120).map(|frame| observation(frame, true)).collect();
    refine_match_events_with_spatial(&mut grounded, &grounded_observations, &context);
    assert_eq!(grounded.jumps[0].direction, JumpDirection::Backward);
    assert_eq!(grounded.jumps[0].outcome, JumpOutcome::GroundedHit);
    assert!(!grounded.jumps[0].takeoff_confirmed);

    let mut airborne = empty_events();
    airborne
        .jumps
        .push(jump(100, JumpOutcome::UnverifiedHit, "UL"));
    let airborne_observations: Vec<_> = (100..=120)
        .map(|frame| observation(frame, frame < 112))
        .collect();
    refine_match_events_with_spatial(&mut airborne, &airborne_observations, &context);
    assert_eq!(airborne.jumps[0].direction, JumpDirection::Forward);
    assert_eq!(airborne.jumps[0].outcome, JumpOutcome::GotHit);
    assert!(airborne.jumps[0].takeoff_confirmed);
}
