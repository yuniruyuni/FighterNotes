use super::*;

#[test]
fn landed_hit_requires_airborne_evidence_until_contact() {
    let actor = |x, ground_anchor| ActorObservation {
        anchor: SpatialPoint::new(x, if ground_anchor { 0.9 } else { 0.72 }),
        bounds: SpatialRect::new(
            x - 0.03,
            0.4,
            x + 0.03,
            if ground_anchor { 0.92 } else { 0.74 },
        ),
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
        contact: None,
        camera: None,
    };
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));

    let mut ground_string = empty_events();
    ground_string
        .jumps
        .push(jump(100, JumpOutcome::LandedHit, "UL"));
    let observations: Vec<_> = (94..=119)
        .map(|frame| {
            // Grounded attack motion briefly exposes only the moving upper body,
            // then the feet are observed well before the hit at f120. A final
            // one-frame upper-body region must not restart an airborne run.
            observation(frame, !(106..=111).contains(&frame) && frame != 119)
        })
        .collect();
    refine_match_events_with_spatial(&mut ground_string, &observations, &context);
    assert_eq!(ground_string.jumps[0].outcome, JumpOutcome::Neutral);
    assert!(!ground_string.jumps[0].takeoff_confirmed);

    let mut jump_in = empty_events();
    jump_in.jumps.push(jump(100, JumpOutcome::LandedHit, "UL"));
    let observations: Vec<_> = (94..=120)
        .map(|frame| observation(frame, frame < 106))
        .collect();
    refine_match_events_with_spatial(&mut jump_in, &observations, &context);
    assert_eq!(jump_in.jumps[0].outcome, JumpOutcome::LandedHit);
    assert!(jump_in.jumps[0].takeoff_confirmed);
}
