use super::*;

#[test]
fn naked_teleport_reach_requires_character_profile_and_overlap() {
    let mut events = empty_events();
    events.teleports.push(teleport(100));
    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observations = vec![SpatialObservation {
        frame_index: 130,
        p1: Some(actor(0.50)),
        p2: Some(actor(0.56)),
        screen_distance: Some(0.06),
        distance_band: Some(DistanceBand::Overlap),
        horizontal_order: Some(HorizontalOrder::Overlapping),
        projectile_candidates: vec![],
        motion_regions: vec![],
    }];

    let ken = AnalysisContext::from_characters("p1", Some("KEN"), Some("DHALSIM"));
    refine_match_events_with_spatial(&mut events, &observations, &ken);
    assert_eq!(
        events.teleports[0].dp_reachability,
        DpReachability::Confirmed
    );

    let mut no_reversal = empty_events();
    no_reversal.teleports.push(teleport(100));
    let zangief = AnalysisContext::from_characters("p1", Some("ZANGIEF"), Some("DHALSIM"));
    refine_match_events_with_spatial(&mut no_reversal, &observations, &zangief);
    assert_eq!(
        no_reversal.teleports[0].dp_reachability,
        DpReachability::Unknown
    );

    let mut charged = empty_events();
    charged.teleports.push(teleport(100));
    charged.segments[0].push(crate::match_events::InputSegment {
        start_frame: 50,
        end_frame: 95,
        dir: "D".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    charged.meter_game_frame[0] = (0..400).map(i64::from).collect();
    let blanka = AnalysisContext::from_characters("p1", Some("BLANKA"), Some("DHALSIM"));
    refine_match_events_with_spatial(&mut charged, &observations, &blanka);
    assert_eq!(
        charged.teleports[0].dp_reachability,
        DpReachability::Confirmed
    );

    let mut uncharged = empty_events();
    uncharged.teleports.push(teleport(100));
    refine_match_events_with_spatial(&mut uncharged, &observations, &blanka);
    assert_eq!(
        uncharged.teleports[0].dp_reachability,
        DpReachability::Unknown
    );

    let mut frozen_charge = empty_events();
    frozen_charge.teleports.push(teleport(100));
    frozen_charge.segments[0].push(crate::match_events::InputSegment {
        start_frame: 50,
        end_frame: 95,
        dir: "D".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    frozen_charge.meter_game_frame[0] = (0..400).map(|frame| i64::from(frame / 2)).collect();
    refine_match_events_with_spatial(&mut frozen_charge, &observations, &blanka);
    assert_eq!(
        frozen_charge.teleports[0].dp_reachability,
        DpReachability::Unknown,
        "映像上45Fでもゲーム内フレームが停止していれば溜め成立としない"
    );
}
