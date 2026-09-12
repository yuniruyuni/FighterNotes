//! 被弾直前の間合いの観測。
//!
//! 差し合いの距離は、技が届く直前に両者がどれだけ離れて立っていたかに
//! 表れる。damage 起点の薄い window で復号した観測から、被弾直前の
//! anchor 間距離を「体の平均身長」単位で測って被弾へ帰属する。画面座標の
//! ままではカメラのズームで揺れるため、同じ画に写る体を物差しにする。

use super::super::parameters::{
    DAMAGE_DISTANCE_LOOKBACK, DAMAGE_DISTANCE_MIN_LEAD, DAMAGE_DISTANCE_MIN_SAMPLES,
    HEIGHT_REFERENCE_MIN_ACTOR_HEIGHT,
};
use super::super::SpatialObservation;
use super::observations::reliable_actor_pair;
use crate::match_events::{DamageDistance, MatchEvents};

pub(super) fn attach(events: &mut MatchEvents, observations: &[SpatialObservation]) {
    let mut distances = Vec::new();
    for damage in &events.damage {
        let start = damage
            .pre_freeze_frame
            .saturating_sub(DAMAGE_DISTANCE_LOOKBACK);
        let end = damage
            .pre_freeze_frame
            .saturating_sub(DAMAGE_DISTANCE_MIN_LEAD);
        let mut samples: Vec<f32> = observations
            .iter()
            .filter(|observation| {
                observation.frame_index >= start && observation.frame_index <= end
            })
            .filter_map(|observation| {
                let (p1, p2) = reliable_actor_pair(observation)?;
                // 空中の体は間合いの物差しにならない(飛び込みは差し合いの
                // 距離ではない)。低すぎる追跡箱も身長単位を歪める。
                if !p1.ground_anchor || !p2.ground_anchor {
                    return None;
                }
                let height =
                    (p1.bounds.bottom - p1.bounds.top + p2.bounds.bottom - p2.bounds.top) / 2.0;
                if height < HEIGHT_REFERENCE_MIN_ACTOR_HEIGHT {
                    return None;
                }
                Some((p1.anchor.x - p2.anchor.x).abs() / height)
            })
            .collect();
        if samples.len() < DAMAGE_DISTANCE_MIN_SAMPLES {
            continue;
        }
        // 1 サンプルの anchor 揺れに引きずられないよう中央値を使う。
        samples.sort_by(f32::total_cmp);
        distances.push(DamageDistance {
            victim: damage.victim,
            damage_start_frame: damage.start_frame,
            distance: samples[samples.len() / 2],
        });
    }
    events.damage_distances = distances;
}
