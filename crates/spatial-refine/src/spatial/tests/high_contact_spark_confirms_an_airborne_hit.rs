use super::*;
use crate::spatial::ContactObservation;

/// 体の追跡が演出で切れていても、hitstop のスパークが立ち姿勢の頭より
/// 上にあれば、空中で接触したことの傍証として GotHit を確認する。
/// スパークの高さ・確度・時刻のどれかが条件を外れれば従来どおり
/// Neutral へ落とす(降格には使わない)。
#[test]
fn high_contact_spark_confirms_an_airborne_hit() {
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));
    // 体の観測は無く、hitstop 帯に contact だけがある観測列を作る。
    let case = |contact_frame: u32, y: f32, confidence: f32| {
        let mut events = empty_events();
        events
            .jumps
            .push(jump(100, JumpOutcome::UnverifiedHit, "UL"));
        let observations: Vec<_> = (94..=132)
            .map(|frame_index| SpatialObservation {
                frame_index,
                p1: None,
                p2: None,
                screen_distance: None,
                distance_band: None,
                horizontal_order: None,
                projectile_candidates: vec![],
                motion_regions: vec![],
                contact: (frame_index == contact_frame).then(|| ContactObservation {
                    center: SpatialPoint::new(0.5, y),
                    bounds: SpatialRect::new(0.45, y - 0.05, 0.55, y + 0.05),
                    effect_cells: 12,
                    confidence,
                }),
                camera: None,
            })
            .collect();
        refine_match_events_with_spatial(&mut events, &observations, &context);
        (events.jumps[0].outcome, events.jumps[0].takeoff_confirmed)
    };

    // contact は f120。hitstop 帯 (120..=130) の高いスパークが確認になる。
    assert_eq!(case(120, 0.30, 0.7), (JumpOutcome::GotHit, true));
    // 帯の末尾ちょうども数え、その 1 フレーム先は数えない。
    assert_eq!(case(130, 0.30, 0.7), (JumpOutcome::GotHit, true));
    assert_eq!(case(131, 0.30, 0.7), (JumpOutcome::Neutral, false));
    // contact より前のスパークは別の接触かもしれない。
    assert_eq!(case(119, 0.30, 0.7), (JumpOutcome::Neutral, false));
    // 高さの境界: 0.42 は立ち姿勢の頭の範囲なので採用しない。
    assert_eq!(case(120, 0.41, 0.7), (JumpOutcome::GotHit, true));
    assert_eq!(case(120, 0.42, 0.7), (JumpOutcome::Neutral, false));
    // 確度の境界: 0.5 ちょうどは採用する。
    assert_eq!(case(120, 0.30, 0.5), (JumpOutcome::GotHit, true));
    assert_eq!(case(120, 0.30, 0.49), (JumpOutcome::Neutral, false));
}

/// 自分のジャンプ攻撃(LandedHit)でも、頭上のスパークは空中証拠の
/// 不足を補って確認済みのまま残す。
#[test]
fn high_contact_spark_preserves_a_landed_hit() {
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));
    let mut events = empty_events();
    events.jumps.push(jump(100, JumpOutcome::LandedHit, "UL"));
    let observations: Vec<_> = (94..=132)
        .map(|frame_index| SpatialObservation {
            frame_index,
            p1: None,
            p2: None,
            screen_distance: None,
            distance_band: None,
            horizontal_order: None,
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: (frame_index == 121).then(|| ContactObservation {
                center: SpatialPoint::new(0.5, 0.35),
                bounds: SpatialRect::new(0.45, 0.30, 0.55, 0.40),
                effect_cells: 12,
                confidence: 0.7,
            }),
            camera: None,
        })
        .collect();
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(events.jumps[0].outcome, JumpOutcome::LandedHit);
    assert!(events.jumps[0].takeoff_confirmed);
}
