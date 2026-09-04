//! 画面端(壁)を背負っている区間の抽出。
//!
//! SF6 のカメラはクランプが無い限り両者の中点を画面中央へ寄せる。
//! したがって中点が中央から大きくずれた画は、壁でパンが止まり片側が
//! 端へ追い込まれていることを意味する。ズームに依存せず、カメラ推定
//! すら要らない、幾何だけの判定になる。
//!
//! 最大ズームアウトでは壁が無くても両者が画面端に写るため、Far バンドの
//! フレームでは判定しない。候補 window 内でしか観測できないので、span が
//! 無いことは「端ではなかった」を意味しない。

use super::super::parameters::{
    CORNER_EDGE_X, CORNER_MAX_GAP, CORNER_MIDPOINT_OFFSET, CORNER_MIN_SAMPLES,
};
use super::super::{DistanceBand, SpatialObservation};
use super::observations::reliable_actor_pair;
use crate::match_events::CornerSpan;

pub(super) fn detect(observations: &[SpatialObservation]) -> Vec<CornerSpan> {
    let mut spans = Vec::new();
    let mut current: Option<(u8, u32, u32, usize)> = None;
    for observation in observations {
        let Some(side) = cornered_side(observation) else {
            continue;
        };
        let frame = observation.frame_index;
        match &mut current {
            Some((span_side, _, end, samples))
                if *span_side == side && frame.saturating_sub(*end) <= CORNER_MAX_GAP =>
            {
                *end = frame;
                *samples += 1;
            }
            _ => {
                flush(&mut spans, current.take());
                current = Some((side, frame, frame, 1));
            }
        }
    }
    flush(&mut spans, current);
    spans
}

fn flush(spans: &mut Vec<CornerSpan>, current: Option<(u8, u32, u32, usize)>) {
    if let Some((side, start_frame, end_frame, samples)) = current {
        if samples >= CORNER_MIN_SAMPLES {
            spans.push(CornerSpan {
                side,
                start_frame,
                end_frame,
            });
        }
    }
}

/// このフレームで端を背負っている側。確認できなければ None。
fn cornered_side(observation: &SpatialObservation) -> Option<u8> {
    let (p1, p2) = reliable_actor_pair(observation)?;
    // 最大ズームアウトの端寄りと壁を混同しない。
    if observation.distance_band? == DistanceBand::Far {
        return None;
    }
    let midpoint = (p1.anchor.x + p2.anchor.x) / 2.0;
    let offset = midpoint - 0.5;
    if offset.abs() < CORNER_MIDPOINT_OFFSET {
        return None;
    }
    // 壁方向への寄りが大きい方が追い込まれている。完全に重なっている
    // 場合は決められないので、規約として P2 とする。
    let toward_wall = |x: f32| (x - 0.5) * offset;
    let (side, wall_x) = if toward_wall(p1.anchor.x) > toward_wall(p2.anchor.x) {
        (1, p1.anchor.x)
    } else {
        (2, p2.anchor.x)
    };
    // 中点の偏りは knockback の土煙などが anchor を流しても起きる。壁の
    // 幾何として、端側の人物が実際に画面端域(壁方向へ 0.5 - 3/16 以上
    // 寄った位置)へ入っていることも要求する。offset の符号で壁の向きを
    // 織り込んだ 1 つの比較にする。
    let near_edge = (wall_x - 0.5) * offset >= (0.5 - CORNER_EDGE_X) * offset.abs();
    near_edge.then_some(side)
}
