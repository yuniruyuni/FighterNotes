use super::*;

#[test]
fn refinement_records_candidate_and_unique_sampled_frames() {
    let mut events = empty_events();
    events.drive_rushes.push(DriveRushEvent {
        side: 1,
        frame: 100,
        raw: true,
        outcome: DriveRushOutcome::NoContact,
        contact_frame: Some(120),
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index: u32, usable: bool| SpatialObservation {
        frame_index,
        p1: usable.then(|| actor(0.4)),
        p2: usable.then(|| actor(0.6)),
        screen_distance: usable.then_some(0.2),
        distance_band: usable.then_some(DistanceBand::Close),
        horizontal_order: usable.then_some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
        contact: None,
        camera: None,
    };

    refine_match_events_with_spatial(
        &mut events,
        [
            observation(85, false),
            observation(86, true),
            observation(86, true),
        ]
        .as_slice(),
        &AnalysisContext::default(),
    );

    assert_eq!(events.spatial_coverage.candidate_frames, 46);
    assert_eq!(events.spatial_coverage.sampled_frames, 2);
    assert_eq!(events.spatial_coverage.usable_frames, 1);
    assert_eq!(events.spatial_coverage.p1_observed_frames, 1);
    assert_eq!(events.spatial_coverage.p2_observed_frames, 1);
}
