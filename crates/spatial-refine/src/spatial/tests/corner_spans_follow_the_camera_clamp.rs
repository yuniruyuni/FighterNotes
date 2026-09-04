use super::*;

/// カメラのクランプで画面中点が片側へ寄った区間を corner span にする。
/// 中点の偏り・バンド・サンプル数・ギャップの各条件を確かめる。
#[test]
fn corner_spans_follow_the_camera_clamp() {
    let actor = |x: f32| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.05, 0.6, x + 0.05, 0.9),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation =
        |frame_index: u32, p1_x: f32, p2_x: f32, band: DistanceBand| SpatialObservation {
            frame_index,
            p1: Some(actor(p1_x)),
            p2: Some(actor(p2_x)),
            screen_distance: Some((p2_x - p1_x).abs()),
            distance_band: Some(band),
            horizontal_order: Some(HorizontalOrder::P1Left),
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        };
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));

    let mut events = empty_events();
    let mut observations = Vec::new();
    // 判定できない中央の場面が先頭にあっても、走査は止まらない。
    observations.push(observation(90, 0.30, 0.70, DistanceBand::Mid));
    // 右壁: 中点 0.75、P2 が壁側。ギャップ 8 フレームまでは同じ span。
    for frame in [100, 104, 112, 120] {
        observations.push(observation(frame, 0.60, 0.90, DistanceBand::Mid));
    }
    // 中点の偏りちょうど 3/32 は端とみなす。0.3125 と 0.875 は 2 進で
    // 正確なので、中点 0.59375 が境界そのものに載る。
    for frame in [200, 201, 202] {
        observations.push(observation(frame, 0.3125, 0.875, DistanceBand::Mid));
    }
    // 中点が偏っていても、端側の人物が画面端域(1 - 3/16 = 0.8125)に
    // 届いていなければ壁ではない(土煙などによる anchor 流れ)。
    for frame in [250, 251, 252] {
        observations.push(observation(frame, 0.45, 0.79, DistanceBand::Mid));
    }
    // 端域の境界ちょうど(0.8125)は壁とみなす。
    for frame in [270, 271, 272] {
        observations.push(observation(frame, 0.4375, 0.8125, DistanceBand::Mid));
    }
    // 左壁でも境界ちょうど(0.1875)は壁とみなす。
    for frame in [280, 281, 282] {
        observations.push(observation(frame, 0.1875, 0.5625, DistanceBand::Mid));
    }
    // 左壁で 2 サンプルだけでは span にしない。
    for frame in [300, 301] {
        observations.push(observation(frame, 0.10, 0.40, DistanceBand::Close));
    }
    // 偏っていても Far バンドは最大ズームアウトの可能性があるので除外。
    for frame in [400, 401, 402] {
        observations.push(observation(frame, 0.45, 0.95, DistanceBand::Far));
    }
    // 中央付近(偏り 0.10 未満)は端ではない。
    for frame in [500, 501, 502] {
        observations.push(observation(frame, 0.25, 0.74, DistanceBand::Mid));
    }
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(
        events.corner_spans,
        vec![
            crate::match_events::CornerSpan {
                side: 2,
                start_frame: 100,
                end_frame: 120,
            },
            crate::match_events::CornerSpan {
                side: 2,
                start_frame: 200,
                end_frame: 202,
            },
            crate::match_events::CornerSpan {
                side: 2,
                start_frame: 270,
                end_frame: 272,
            },
            crate::match_events::CornerSpan {
                side: 1,
                start_frame: 280,
                end_frame: 282,
            },
        ]
    );

    // 両者が完全に重なっているときは規約で P2 を端とする。
    let mut events = empty_events();
    let observations: Vec<_> = [600, 601, 602]
        .iter()
        .map(|&frame| observation(frame, 0.875, 0.875, DistanceBand::Overlap))
        .collect();
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(
        events.corner_spans,
        vec![crate::match_events::CornerSpan {
            side: 2,
            start_frame: 600,
            end_frame: 602,
        }]
    );

    // 左壁は P1 が背負う。ギャップ 9 フレームで span は 2 つに切れる。
    let mut events = empty_events();
    let observations: Vec<_> = [100, 101, 102, 111, 112, 113]
        .iter()
        .map(|&frame| observation(frame, 0.10, 0.40, DistanceBand::Close))
        .collect();
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(
        events.corner_spans,
        vec![
            crate::match_events::CornerSpan {
                side: 1,
                start_frame: 100,
                end_frame: 102,
            },
            crate::match_events::CornerSpan {
                side: 1,
                start_frame: 111,
                end_frame: 113,
            },
        ]
    );
}
