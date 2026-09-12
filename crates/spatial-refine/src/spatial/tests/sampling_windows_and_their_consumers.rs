use super::*;
use crate::match_events::{DamageDistance, DamageEvent};

fn actor(x: f32) -> ActorObservation {
    ActorObservation {
        anchor: SpatialPoint::new(x, 0.875),
        // 身長 1/2(頭 3/8、足元 7/8)。2 進で正確に割れる物差し。
        bounds: SpatialRect::new(x - 0.05, 0.375, x + 0.05, 0.875),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    }
}

fn observation(frame_index: u32, p1_x: f32, p2_x: f32) -> SpatialObservation {
    SpatialObservation {
        frame_index,
        p1: Some(actor(p1_x)),
        p2: Some(actor(p2_x)),
        screen_distance: Some((p2_x - p1_x).abs()),
        distance_band: Some(DistanceBand::Mid),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
        contact: None,
        camera: None,
    }
}

fn damage(victim: u8, start_frame: u32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 10,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }
}

/// 復号計画は、イベント駆動の候補に薄いサンプリング window を加える。
/// 定周期 window は round 境界の外を覗かず、damage window は被弾直前
/// (演出フリーズがあればその前)だけを遡る。
#[test]
fn the_decode_plan_adds_sampling_windows_inside_rounds() {
    let mut events = empty_events();
    events.rounds[0].end_frame = 480;
    // 終端が周期の位相ちょうどに載る round。境界の window を落とさない。
    let mut second_round = events.rounds[0].clone();
    second_round.round_no = 2;
    second_round.start_frame = 500;
    second_round.end_frame = 740;
    events.rounds.push(second_round);
    events.damage.push(DamageEvent {
        pre_freeze_frame: 190,
        ..damage(1, 200)
    });
    // round 終端を超える被弾直前は round までで切る。
    events.damage.push(DamageEvent {
        pre_freeze_frame: 500,
        ..damage(1, 510)
    });

    let ranges = |windows: &[SpatialCandidateWindow]| {
        windows
            .iter()
            .map(|window| (window.start_frame, window.end_frame))
            .collect::<Vec<_>>()
    };
    // イベント駆動の候補には sampling は混ざらない。
    assert!(ranges(&crate::spatial_candidate_windows(&events)).is_empty());
    // round は 0..=480。定周期は 0,240 と、round 終端ちょうどに載る
    // 480 起点。damage は pre_freeze から 24F 遡り、round 終端を超える分は
    // 切られて 480 起点の周期 window と統合される。第 2 round(500..=740)の
    // 定周期は 500 と、終端ちょうどの 740。
    assert_eq!(
        ranges(&crate::spatial_decode_windows(&events)),
        [
            (0, 5),
            (166, 190),
            (240, 245),
            (476, 480),
            (500, 505),
            (740, 740)
        ]
    );
}

/// 被弾直前の間合いは、地上の両者の anchor 間距離を平均身長で割った
/// 単位で被弾へ帰属する。1 サンプルでは断定せず、中央値を使う。
#[test]
fn damage_distances_use_the_median_of_grounded_samples() {
    let mut events = empty_events();
    events.damage.push(damage(1, 200));

    // 1 サンプルしか観測できなかった被弾は帰属せず、続く被弾の探索は
    // 止まらない。
    events.damage.insert(0, damage(1, 100));

    // pre_freeze 200 の遡り窓 [176, 198]。距離と身長は 2 進で正確な値に
    // して、0.5/0.5 = 1.0、0.75/0.5 = 1.5、0.5/0.5 = 1.0 の中央値 1.0 を
    // 境界まで等値で確かめる。
    let mut observations = vec![
        observation(90, 0.25, 0.75),
        observation(180, 0.25, 0.75),
        observation(185, 0.125, 0.875),
        observation(190, 0.25, 0.75),
    ];
    // 窓の外(接触に近すぎる 199 と、遡りすぎの 175)は使わない。
    observations.push(observation(175, 0.125, 0.9375));
    observations.push(observation(199, 0.4375, 0.5625));

    refine_match_events_with_spatial(&mut events, &observations, &test_context());
    assert_eq!(
        events.damage_distances,
        vec![DamageDistance {
            victim: 1,
            damage_start_frame: 200,
            distance: 1.0,
        }]
    );

    // 最低サンプル数(2)ちょうどでは帰属し、中央値は大きい側を取る
    // (1.0 と 1.5 なら 1.5)。
    let mut events = empty_events();
    events.damage.push(damage(1, 200));
    let two_samples = vec![observation(180, 0.25, 0.75), observation(185, 0.125, 0.875)];
    refine_match_events_with_spatial(&mut events, &two_samples, &test_context());
    assert_eq!(
        events.damage_distances,
        vec![DamageDistance {
            victim: 1,
            damage_start_frame: 200,
            distance: 1.5,
        }]
    );

    // 空中の体は間合いの物差しにならない。地上サンプルが 1 つだけに
    // なると、揺れから断定しないため帰属しない。
    let mut airborne = observations.clone();
    for observation in airborne.iter_mut().skip(1).take(2) {
        if let Some(p1) = observation.p1.as_mut() {
            p1.ground_anchor = false;
        }
    }
    let mut events = empty_events();
    events.damage.push(damage(1, 200));
    refine_match_events_with_spatial(&mut events, &airborne, &test_context());
    assert_eq!(events.damage_distances, vec![]);
}

/// 端滞在時間の分母は定周期サンプルの位相に載ったフレームだけ。位相の
/// 外のフレームは、攻防へ偏ったイベント駆動 window の観測なので使わない。
#[test]
fn corner_time_counts_only_periodic_phase_samples() {
    let mut events = empty_events();
    let mut observations = Vec::new();
    // 位相内でも両者を追跡できないフレームは分母に入らない。後続の
    // 集計は止まらない。
    let mut unreliable = observation(1, 0.60, 0.90);
    unreliable.p2 = None;
    observations.push(unreliable);
    // 位相内(0..=5, 240..=245)。P2 が右壁を背負っている。
    for frame in [0, 2, 4, 240, 242] {
        observations.push(observation(frame, 0.60, 0.90));
    }
    // 位相内だが端ではない中央の場面。分母にだけ入る。
    observations.push(observation(244, 0.375, 0.625));
    // 位相の外は端でも数えない。
    observations.push(observation(60, 0.60, 0.90));
    // round の外は位相が合っていても数えない。
    observations.push(observation(480, 0.60, 0.90));

    refine_match_events_with_spatial(&mut events, &observations, &test_context());
    assert_eq!(events.spatial_coverage.periodic_pair_samples, 6);
    assert_eq!(events.spatial_coverage.p1_cornered_samples, 0);
    assert_eq!(events.spatial_coverage.p2_cornered_samples, 5);
}

fn test_context() -> AnalysisContext {
    AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"))
}
