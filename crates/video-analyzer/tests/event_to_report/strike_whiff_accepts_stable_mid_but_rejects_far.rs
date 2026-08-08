use spatial_refine::test_support::*;
use video_analyzer::advice;
use video_analyzer::context::AnalysisContext;
use video_analyzer::match_events::{PunishChance, PunishOrigin, PunishOutcome, PunishReachability};
use video_analyzer::spatial::{
    refine_match_events_with_spatial, ActorObservation, DistanceBand, HorizontalOrder,
    SpatialObservation, SpatialPoint, SpatialRect,
};

#[test]
fn strike_whiff_accepts_stable_mid_but_rejects_far() {
    let actor = |x| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index, band, p2_x| SpatialObservation {
        frame_index,
        p1: Some(actor(0.35)),
        p2: Some(actor(p2_x)),
        screen_distance: Some((p2_x - 0.35).abs()),
        distance_band: Some(band),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
    };
    let candidate = || PunishChance {
        frame: 200,
        side: 2,
        advantage: 7,
        outcome: PunishOutcome::WhiffFail,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 194,
        recovery_end_frame: 207,
        source_contact_frame: Some(193),
        attack_start_frame: Some(200),
        attack_active_frame: Some(205),
        reachability: PunishReachability::Unknown,
        punished_drop: 0.14,
        pressed: "弱".to_string(),
        round_no: 1,
    };
    let context = AnalysisContext::from_characters("p2", Some("LUKE"), Some("KEN"));

    let mut mid = empty_events();
    mid.punishes.push(candidate());
    let mid_observations: Vec<_> = (157..=213)
        .map(|frame| observation(frame, DistanceBand::Mid, 0.65))
        .collect();
    refine_match_events_with_spatial(&mut mid, &mid_observations, &context);
    assert_eq!(mid.punishes[0].reachability, PunishReachability::Confirmed);
    let card = advice::detect_punish_fail(&mid, 2, Some("LUKE"))
        .expect("安定した距離確認後だけ確反失敗を提示する");
    assert_eq!(card.evidence[0].frame, 200);
    assert!(card.evidence[0].label.contains("距離確認"));

    let mut far = empty_events();
    far.punishes.push(candidate());
    refine_match_events_with_spatial(
        &mut far,
        &[
            observation(193, DistanceBand::Far, 0.90),
            observation(204, DistanceBand::Far, 0.90),
        ],
        &context,
    );
    assert_eq!(far.punishes[0].reachability, PunishReachability::OutOfRange);
}
