use super::*;
use crate::match_events::{DriveRushEvent, DriveRushOutcome, InputSegment};
use crate::spatial::CameraMotion;

fn actor(x: f32) -> ActorObservation {
    ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.9),
        confidence: 1.0,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    }
}

fn observation(frame_index: u32, distance: f32, zoom_ratio: f32) -> SpatialObservation {
    SpatialObservation {
        frame_index,
        p1: Some(actor(0.4)),
        p2: Some(actor(0.4 + distance)),
        screen_distance: Some(distance),
        distance_band: Some(DistanceBand::Mid),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
        contact: None,
        camera: Some(CameraMotion {
            pan_dx: 0.0,
            zoom_ratio,
            confidence: 0.9,
        }),
    }
}

fn rush(frame: u32) -> DriveRushEvent {
    DriveRushEvent {
        side: 1,
        frame,
        raw: true,
        outcome: DriveRushOutcome::Unconfirmed,
        contact_frame: Some(frame + 30),
        damage: 0.0,
        confidence: EventConfidence::Low,
        round_no: 1,
    }
}

fn forward_segment(frame: u32) -> InputSegment {
    InputSegment {
        start_frame: frame,
        end_frame: frame + 20,
        dir: "R".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

/// SF6 のカメラは接近でズームインするため、screen 距離では前進が縮んで
/// 見える。ズーム補正した距離で詰めを判定する。
#[test]
fn zoom_correction_reveals_hidden_approach() {
    let context = AnalysisContext::from_characters("p1", Some("CHUN_LI"), Some("LUKE"));

    // screen 距離は一定 0.30 のまま、毎フレーム 1% ズームイン。
    // 補正後は 0.30 → 0.30/1.01^20 ≈ 0.246 で 0.04 以上詰めている。
    let mut events = empty_events();
    events.drive_rushes.push(rush(100));
    events.segments[0].push(forward_segment(100));
    let observations: Vec<_> = (100..=120)
        .map(|frame| observation(frame, 0.30, if frame == 100 { 1.0 } else { 1.01 }))
        .collect();
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(events.drive_rushes[0].confidence, EventConfidence::High);

    // 逆に、screen 距離が 0.05 縮んでもズームアウトで説明できるなら
    // 前進とは呼ばない(補正後はむしろ離れている)。
    let mut events = empty_events();
    events.drive_rushes.push(rush(100));
    events.segments[0].push(forward_segment(100));
    let observations: Vec<_> = (100..=120)
        .map(|frame| {
            let progress = (frame - 100) as f32 / 20.0;
            observation(
                frame,
                0.35 - 0.05 * progress,
                if frame == 100 { 1.0 } else { 0.99 },
            )
        })
        .collect();
    refine_match_events_with_spatial(&mut events, &observations, &context);
    assert_eq!(events.drive_rushes[0].confidence, EventConfidence::Low);
}
