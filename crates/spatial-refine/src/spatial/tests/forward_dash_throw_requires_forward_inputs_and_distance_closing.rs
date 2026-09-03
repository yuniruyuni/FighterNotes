use super::*;

#[test]
fn forward_dash_throw_requires_forward_inputs_and_distance_closing() {
    use crate::match_events::{InputSegment, ThrowActionEvent};
    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index, p1_x, band| SpatialObservation {
        frame_index,
        p1: Some(actor(p1_x)),
        p2: Some(actor(0.62)),
        screen_distance: Some((0.62 - p1_x).abs()),
        distance_band: Some(band),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
        contact: None,
        camera: None,
    };
    let mut events = empty_events();
    events.throw_actions.push(ThrowActionEvent {
        thrower: 1,
        input_frame: 100,
        startup_frame: Some(102),
        active_frame: Some(107),
        outcome: ThrowOutcome::Hit,
        damage: 0.12,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    let direction = |frame, dir: &str| InputSegment {
        start_frame: frame,
        end_frame: frame + 2,
        dir: dir.to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    events.segments[0] = vec![direction(65, "R"), direction(78, "R")];
    let observations = vec![
        observation(60, 0.25, DistanceBand::Mid),
        observation(82, 0.42, DistanceBand::Close),
        observation(107, 0.54, DistanceBand::Close),
    ];
    refine_match_events_with_spatial(
        &mut events,
        &observations,
        &AnalysisContext::from_characters("p1", Some("LUKE"), Some("KEN")),
    );
    assert_eq!(events.throw_actions[0].approach, ThrowApproach::ForwardDash);

    let mut backward = empty_events();
    backward.throw_actions = events.throw_actions.clone();
    backward.throw_actions[0].approach = ThrowApproach::Unknown;
    backward.segments[0] = vec![direction(65, "L"), direction(78, "L")];
    refine_match_events_with_spatial(
        &mut backward,
        &observations,
        &AnalysisContext::from_characters("p1", Some("LUKE"), Some("KEN")),
    );
    assert_eq!(backward.throw_actions[0].approach, ThrowApproach::Unknown);
}
