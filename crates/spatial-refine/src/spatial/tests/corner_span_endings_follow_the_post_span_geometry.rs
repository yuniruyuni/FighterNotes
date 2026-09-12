use super::*;
use crate::match_events::CornerEnding;

/// span 終端後の幾何から終わり方を分類する。左右の入れ替えは SideSwap、
/// 中点の偏りが解けるまで離れたら Separated、直後の観測が無い・確認に
/// 足りない場合は Unobserved のまま終わり方を語らない。
#[test]
fn corner_span_endings_follow_the_post_span_geometry() {
    let actor = |x: f32| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.05, 0.6, x + 0.05, 0.9),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    let observation = |frame_index: u32, p1_x: f32, p2_x: f32| SpatialObservation {
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
    };
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));
    // 右壁に P2 を追い込んだ span を作る 3 サンプル。
    let span =
        |start: u32| [start, start + 2, start + 4].map(|frame| observation(frame, 0.60, 0.90));
    let endings = |observations: &[SpatialObservation]| {
        let mut events = empty_events();
        refine_match_events_with_spatial(&mut events, observations, &context);
        events
            .corner_spans
            .iter()
            .map(|span| span.ending)
            .collect::<Vec<_>>()
    };

    // 入れ替え: P2 が P1 の左(壁と反対側)で 3 回観測された。
    let mut swapped: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    for frame in [140, 150, 160] {
        swapped.push(observation(frame, 0.60, 0.40));
    }
    assert_eq!(endings(&swapped), vec![CornerEnding::SideSwap]);

    // 離脱: 左右そのままに中点の偏りが解けた(壁から離れた)。
    let mut separated: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    for frame in [140, 150, 160] {
        separated.push(observation(frame, 0.35, 0.65));
    }
    assert_eq!(endings(&separated), vec![CornerEnding::Separated]);

    // 入れ替えの気配(1 サンプル)が混ざった離脱は断定しない。overlap の
    // 揺れで anchor が入れ替わって見える瞬間があるため。
    let mut mixed: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    mixed.push(observation(140, 0.60, 0.55));
    for frame in [150, 160, 170] {
        mixed.push(observation(frame, 0.35, 0.65));
    }
    assert_eq!(endings(&mixed), vec![CornerEnding::Unobserved]);

    // 確認が 2 サンプルでは足りない。
    let mut short: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    for frame in [140, 150] {
        short.push(observation(frame, 0.35, 0.65));
    }
    assert_eq!(endings(&short), vec![CornerEnding::Unobserved]);

    // 終端から 90F を超えた観測は別の場面。境界ちょうど(+90)は数える。
    let mut at_boundary: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    for frame in [150, 170, 194] {
        at_boundary.push(observation(frame, 0.35, 0.65));
    }
    assert_eq!(endings(&at_boundary), vec![CornerEnding::Separated]);
    let mut past_boundary: Vec<_> = span(100).into_iter().collect::<Vec<_>>();
    for frame in [150, 170, 195] {
        past_boundary.push(observation(frame, 0.35, 0.65));
    }
    assert_eq!(endings(&past_boundary), vec![CornerEnding::Unobserved]);

    // 左壁(P1 が壁側)でも鏡像で判定する。P1 が P2 の右へ回れば入れ替え。
    let mut left_wall: Vec<_> = [100, 102, 104]
        .map(|frame| observation(frame, 0.10, 0.40))
        .into_iter()
        .collect::<Vec<_>>();
    for frame in [140, 150, 160] {
        left_wall.push(observation(frame, 0.60, 0.40));
    }
    assert_eq!(endings(&left_wall), vec![CornerEnding::SideSwap]);
}
