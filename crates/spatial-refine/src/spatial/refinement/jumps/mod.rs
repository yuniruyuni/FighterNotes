mod direction;
mod samples;

use super::super::parameters::{
    CONTACT_AIRBORNE_MAX_Y, CONTACT_AIRBORNE_MIN_CONFIDENCE, CONTACT_HEIGHT_DEFENDER_LOOKBACK,
    CONTACT_HEIGHT_MIN_DEFENDER_HEIGHT, CONTACT_HINT_TAIL_FRAMES, JUMP_AIR_SAMPLE_LOOKBACK,
    JUMP_SPATIAL_LOOKAHEAD, JUMP_SPATIAL_LOOKBACK,
};
use super::super::SpatialObservation;
use super::observations::reliable_actor_pair;
use crate::match_events::{JumpDirection, JumpEvent, JumpOutcome};

pub(super) fn refine(jumps: &mut [JumpEvent], observations: &[SpatialObservation]) {
    for jump in jumps {
        refine_one(jump, observations);
    }
}

fn refine_one(jump: &mut JumpEvent, observations: &[SpatialObservation]) {
    if jump.direction == JumpDirection::Unknown {
        let order = observations
            .iter()
            .filter(|observation| observation.frame_index.abs_diff(jump.frame) <= 4)
            .find_map(|observation| {
                reliable_actor_pair(observation)?;
                observation.horizontal_order
            });
        jump.direction = direction::resolve(jump.side, &jump.input_dir, order);
    }

    if !matches!(
        jump.outcome,
        JumpOutcome::GotHit | JumpOutcome::UnverifiedHit | JumpOutcome::LandedHit
    ) {
        return;
    }
    let Some(contact_frame) = jump.contact_frame else {
        return;
    };
    let coverage_start = jump.frame.saturating_sub(JUMP_SPATIAL_LOOKBACK);
    let coverage_end = contact_frame.saturating_add(JUMP_SPATIAL_LOOKAHEAD);
    if !observations.iter().any(|observation| {
        observation.frame_index >= coverage_start && observation.frame_index <= coverage_end
    }) {
        // Refinement may receive observations for an unrelated event window.
        // Leave jumps that were not sampled untouched.
        return;
    }
    let sample_start = if jump.outcome == JumpOutcome::LandedHit {
        coverage_start
    } else {
        contact_frame.saturating_sub(JUMP_AIR_SAMPLE_LOOKBACK)
    };
    let actor_samples =
        samples::actor_samples(observations, jump.side, sample_start, contact_frame);
    let high_spark = high_contact_spark(observations, contact_frame);
    if jump.outcome == JumpOutcome::LandedHit {
        samples::refine_landed_hit(jump, &actor_samples, contact_frame, high_spark);
    } else {
        samples::refine_incoming_hit(jump, &actor_samples, high_spark);
    }
    // 高さは確定した空中接触にだけ意味を持つ。Neutral へ降格した候補に
    // 残すと、成立しなかったジャンプが高さ統計へ混ざる。
    if matches!(jump.outcome, JumpOutcome::GotHit | JumpOutcome::LandedHit) {
        jump.contact_height = contact_height(observations, jump.side, contact_frame);
    }
}

/// 接触スパークの高さを、地上にいる防御側の身長単位(足元 0、頭 1)へ
/// 正規化する。画面座標そのままではカメラのズームに揺れるため、同じ画に
/// 写っている防御側の体を物差しにする。
fn contact_height(
    observations: &[SpatialObservation],
    jumper_side: u8,
    contact_frame: u32,
) -> Option<f32> {
    let spark = observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= contact_frame
                && observation.frame_index <= contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES)
        })
        .filter_map(|observation| observation.contact.as_ref())
        .filter(|contact| contact.confidence >= CONTACT_AIRBORNE_MIN_CONFIDENCE)
        .max_by(|a, b| a.confidence.total_cmp(&b.confidence))?;
    let defender = 3 - jumper_side;
    let sample_start = contact_frame.saturating_sub(CONTACT_HEIGHT_DEFENDER_LOOKBACK);
    let sample_end = contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES);
    let defender_samples = samples::actor_samples(observations, defender, sample_start, sample_end);
    let (_, standing) = defender_samples
        .iter()
        .rev()
        .find(|(_, actor)| actor.ground_anchor)?;
    let defender_height = standing.bounds.bottom - standing.bounds.top;
    if defender_height < CONTACT_HEIGHT_MIN_DEFENDER_HEIGHT {
        return None;
    }
    Some((standing.bounds.bottom - spark.center.y) / defender_height)
}

/// hitstop 中のスパークが立ち姿勢の頭より明確に上にあれば、その接触は
/// 空中の身体に当たっている。体の追跡が演出で切れた場面の傍証になる。
fn high_contact_spark(observations: &[SpatialObservation], contact_frame: u32) -> bool {
    observations
        .iter()
        .filter(|observation| {
            observation.frame_index >= contact_frame
                && observation.frame_index <= contact_frame.saturating_add(CONTACT_HINT_TAIL_FRAMES)
        })
        .filter_map(|observation| observation.contact.as_ref())
        .any(|contact| {
            contact.confidence >= CONTACT_AIRBORNE_MIN_CONFIDENCE
                && contact.center.y < CONTACT_AIRBORNE_MAX_Y
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{
        ActorObservation, DistanceBand, HorizontalOrder, SpatialPoint, SpatialRect,
    };

    fn actor() -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(0.3, 0.9),
            bounds: SpatialRect::new(0.27, 0.5, 0.33, 0.9),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    #[test]
    fn an_observation_before_the_exact_coverage_start_does_not_refine_the_jump() {
        let mut jumps = [JumpEvent {
            side: 1,
            frame: 10,
            outcome: JumpOutcome::GotHit,
            input_dir: "U".into(),
            direction: JumpDirection::Neutral,
            contact_frame: Some(20),
            takeoff_confirmed: true,
            air_end: 40,
            contact_height: None,
            round_no: 1,
        }];
        let observations = [SpatialObservation {
            frame_index: 3,
            p1: Some(actor()),
            p2: Some(actor()),
            screen_distance: Some(0.2),
            distance_band: Some(DistanceBand::Close),
            horizontal_order: Some(HorizontalOrder::P1Left),
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        }];

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::GotHit);
        assert!(jumps[0].takeoff_confirmed);
    }

    use crate::spatial::ContactObservation;

    fn observation(frame_index: u32) -> SpatialObservation {
        SpatialObservation {
            frame_index,
            p1: None,
            p2: None,
            screen_distance: None,
            distance_band: None,
            horizontal_order: None,
            projectile_candidates: vec![],
            motion_regions: vec![],
            contact: None,
            camera: None,
        }
    }

    /// 足元 7/8、頭 3/8 の立ち姿勢。身長 1/2 で、2 進で正確に割れる。
    fn standing(bottom: f32, top: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(0.6, bottom),
            bounds: SpatialRect::new(0.57, top, 0.63, bottom),
            confidence: 1.0,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    fn airborne_jumper() -> ActorObservation {
        ActorObservation {
            ground_anchor: false,
            anchor: SpatialPoint::new(0.4, 0.5),
            bounds: SpatialRect::new(0.37, 0.3, 0.43, 0.5),
            ..standing(0.9, 0.5)
        }
    }

    fn spark(y: f32, confidence: f32) -> ContactObservation {
        ContactObservation {
            center: SpatialPoint::new(0.45, y),
            bounds: SpatialRect::new(0.42, y - 0.03, 0.48, y + 0.03),
            effect_cells: 8,
            confidence,
        }
    }

    fn incoming_jump(contact_frame: u32) -> JumpEvent {
        JumpEvent {
            side: 2,
            frame: 10,
            outcome: JumpOutcome::GotHit,
            input_dir: "U".into(),
            direction: JumpDirection::Neutral,
            contact_frame: Some(contact_frame),
            takeoff_confirmed: true,
            air_end: 60,
            contact_height: None,
            round_no: 1,
        }
    }

    /// スパーク高さは、地上の防御側の身長単位へ正規化して記録する。
    /// 画面座標そのままではカメラのズームに揺れるため、同じ画に写る
    /// 防御側の体を物差しにする。1.25 = 高い位置の境界そのもの。
    #[test]
    fn contact_height_is_normalized_by_the_grounded_defender() {
        let mut jumps = [incoming_jump(20)];
        let mut observations = vec![observation(14), observation(16), observation(20)];
        for frame in &mut observations[0..2] {
            frame.p1 = Some(standing(0.875, 0.375));
            frame.p2 = Some(airborne_jumper());
        }
        observations[2].contact = Some(spark(0.25, 0.8));

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::GotHit);
        assert_eq!(jumps[0].contact_height, Some(1.25), "(0.875-0.25)/0.5");
    }

    /// 低確度のスパーク、低すぎる防御側の追跡箱、接地サンプルの欠落は、
    /// どれも身長単位の物差しにならないので高さを記録しない。
    #[test]
    fn unreliable_sparks_and_defender_boxes_yield_no_height() {
        let with = |spark_conf: f32, defender: Option<ActorObservation>| {
            let mut jumps = [incoming_jump(20)];
            let mut observations = vec![observation(14), observation(16), observation(20)];
            for frame in &mut observations[0..2] {
                frame.p1 = defender.clone();
                frame.p2 = Some(airborne_jumper());
            }
            observations[2].contact = Some(spark(0.25, spark_conf));
            refine(&mut jumps, &observations);
            (jumps[0].outcome, jumps[0].contact_height)
        };

        // スパークの確度が足りない。
        assert_eq!(
            with(0.49, Some(standing(0.875, 0.375))),
            (JumpOutcome::GotHit, None)
        );
        // 防御側の箱が最低身長(1/8)を割る。しゃがみ・部分追跡の可能性。
        assert_eq!(
            with(0.8, Some(standing(0.875, 0.8))),
            (JumpOutcome::GotHit, None)
        );
        // ちょうど最低身長なら物差しとして使う。(0.875-0.25)/0.125 = 5。
        assert_eq!(
            with(0.8, Some(standing(0.875, 0.75))),
            (JumpOutcome::GotHit, Some(5.0))
        );
        // 防御側の接地サンプルが無い。
        assert_eq!(with(0.8, None), (JumpOutcome::GotHit, None));
    }

    /// 防御側の接地サンプルは contact の 20F 前までしか遡らない。それより
    /// 古い立ち姿勢は、歩き・しゃがみで既に別の姿勢になっている可能性が
    /// 高く、身長の物差しとして信用できない。
    #[test]
    fn a_stale_defender_sample_is_not_a_height_reference() {
        let mut jumps = [JumpEvent {
            frame: 50,
            contact_frame: Some(60),
            air_end: 100,
            ..incoming_jump(60)
        }];
        // 防御側の接地は境界より 1F 古い(60 - 20 = 40 の窓の外)。
        let mut observations = vec![
            observation(39),
            observation(54),
            observation(56),
            observation(60),
        ];
        observations[0].p1 = Some(standing(0.875, 0.375));
        for frame in &mut observations[1..3] {
            frame.p2 = Some(airborne_jumper());
        }
        observations[3].contact = Some(spark(0.25, 0.8));

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::GotHit);
        assert_eq!(jumps[0].contact_height, None);

        // 窓の内側なら物差しになる。
        jumps[0].contact_height = None;
        jumps[0].outcome = JumpOutcome::GotHit;
        observations[0].frame_index = 40;

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].contact_height, Some(1.25));
    }

    /// Neutral へ降格した候補には高さを残さない。成立しなかったジャンプが
    /// 高さ統計へ混ざると、対空の質の分母が壊れる。
    #[test]
    fn a_downgraded_jump_keeps_no_contact_height() {
        let mut jumps = [incoming_jump(20)];
        let mut observations = vec![observation(14), observation(16), observation(20)];
        for frame in &mut observations[0..2] {
            frame.p1 = Some(standing(0.875, 0.375));
            // 跳んだ側の空中サンプルが無い(接地もしていない)。
            frame.p2 = None;
        }
        // 頭より下のスパークなので空中接触の傍証にもならない。
        observations[2].contact = Some(spark(0.5, 0.8));

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::Neutral);
        assert_eq!(jumps[0].contact_height, None);
    }

    /// 自分の飛び込み(LandedHit)でも同じ物差しで高さが付く。防御側は
    /// 跳んでいない側なので、side に応じて反対側の体を使う。
    #[test]
    fn a_landed_hit_measures_against_the_other_side() {
        let mut jumps = [JumpEvent {
            side: 1,
            ..incoming_jump(20)
        }];
        jumps[0].outcome = JumpOutcome::LandedHit;
        let mut observations = vec![observation(14), observation(18), observation(20)];
        for frame in &mut observations[0..2] {
            frame.p1 = Some(airborne_jumper());
            frame.p2 = Some(standing(0.875, 0.375));
        }
        observations[2].contact = Some(spark(0.375, 0.8));

        refine(&mut jumps, &observations);

        assert_eq!(jumps[0].outcome, JumpOutcome::LandedHit);
        assert_eq!(jumps[0].contact_height, Some(1.0), "(0.875-0.375)/0.5");
    }
}
