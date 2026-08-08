use super::*;

#[test]
fn missed_punish_reach_requires_overlap() {
    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index, distance_band, p2_x| SpatialObservation {
        frame_index,
        p1: Some(actor(0.40)),
        p2: Some(actor(p2_x)),
        screen_distance: Some((p2_x - 0.40).abs()),
        distance_band: Some(distance_band),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
    };
    let candidate = || PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    };
    let context = AnalysisContext::from_characters("p1", Some("BLANKA"), Some("DHALSIM"));

    let mut overlap = empty_events();
    overlap.punishes.push(candidate());
    refine_match_events_with_spatial(
        &mut overlap,
        &[
            observation(195, DistanceBand::Overlap, 0.46),
            observation(196, DistanceBand::Overlap, 0.46),
        ],
        &context,
    );
    assert_eq!(
        overlap.punishes[0].reachability,
        PunishReachability::Confirmed
    );

    let mut mixed = empty_events();
    mixed.punishes.push(candidate());
    refine_match_events_with_spatial(
        &mut mixed,
        &[
            observation(195, DistanceBand::Overlap, 0.46),
            observation(196, DistanceBand::Overlap, 0.46),
            observation(197, DistanceBand::Mid, 0.67),
        ],
        &context,
    );
    assert_eq!(
        mixed.punishes[0].reachability,
        PunishReachability::Unknown,
        "距離帯が安定しない Missed は断定しない"
    );

    let mut far = empty_events();
    far.punishes.push(candidate());
    refine_match_events_with_spatial(
        &mut far,
        &[
            observation(195, DistanceBand::Far, 0.80),
            observation(196, DistanceBand::Far, 0.80),
        ],
        &context,
    );
    assert_eq!(far.punishes[0].reachability, PunishReachability::OutOfRange);

    let mut close = empty_events();
    close.punishes.push(candidate());
    refine_match_events_with_spatial(
        &mut close,
        &[
            observation(195, DistanceBand::Close, 0.58),
            observation(196, DistanceBand::Close, 0.58),
        ],
        &context,
    );
    assert_eq!(
        close.punishes[0].reachability,
        PunishReachability::Unknown,
        "技ごとのリーチが無い Close は断定しない"
    );
}
