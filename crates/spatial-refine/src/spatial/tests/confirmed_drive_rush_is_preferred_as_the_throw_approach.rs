use super::*;

#[test]
fn confirmed_drive_rush_is_preferred_as_the_throw_approach() {
    use crate::match_events::{InputSegment, ThrowActionEvent, ThrowOutcome};

    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index, p1_x| SpatialObservation {
        frame_index,
        p1: Some(actor(p1_x)),
        p2: Some(actor(0.65)),
        screen_distance: Some(0.65 - p1_x),
        distance_band: Some(DistanceBand::Close),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
        contact: None,
        camera: None,
    };
    let mut events = empty_events();
    events.drive_rushes.push(DriveRushEvent {
        side: 1,
        frame: 50,
        raw: true,
        outcome: DriveRushOutcome::Blocked,
        contact_frame: Some(100),
        damage: 0.0,
        confidence: EventConfidence::Medium,
        round_no: 1,
    });
    events.throw_actions.push(ThrowActionEvent {
        thrower: 1,
        input_frame: 105,
        startup_frame: Some(106),
        active_frame: Some(110),
        outcome: ThrowOutcome::Hit,
        damage: 0.12,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.segments[0].push(InputSegment {
        start_frame: 50,
        end_frame: 52,
        dir: "R".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    let observations = [observation(42, 0.25), observation(100, 0.55)];

    refine_match_events_with_spatial(&mut events, &observations, &AnalysisContext::default());

    assert_eq!(events.drive_rushes[0].confidence, EventConfidence::High);
    assert_eq!(events.throw_actions[0].approach, ThrowApproach::DriveRush);
}
