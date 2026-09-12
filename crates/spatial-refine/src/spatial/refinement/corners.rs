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
    CORNER_EDGE_X, CORNER_END_LOOKAHEAD, CORNER_MAX_GAP, CORNER_MIDPOINT_OFFSET, CORNER_MIN_SAMPLES,
};
use super::super::{DistanceBand, SpatialObservation};
use super::observations::reliable_actor_pair;
use crate::match_events::{CornerEnding, CornerSpan};

/// 組み立て中の span。wall_sign は壁の向き(中点の偏りの符号。
/// 正なら右の壁)で、終わり方の分類に使う。
struct BuildingSpan {
    side: u8,
    start_frame: u32,
    end_frame: u32,
    samples: usize,
    wall_sign: f32,
}

pub(super) fn detect(observations: &[SpatialObservation]) -> Vec<CornerSpan> {
    let mut spans = Vec::new();
    let mut current: Option<BuildingSpan> = None;
    for observation in observations {
        let Some((side, offset)) = cornered_side(observation) else {
            continue;
        };
        let frame = observation.frame_index;
        match &mut current {
            Some(span)
                if span.side == side && frame.saturating_sub(span.end_frame) <= CORNER_MAX_GAP =>
            {
                // 同じ側が連続して端を背負っている間、壁の向きは変わらない。
                span.end_frame = frame;
                span.samples += 1;
            }
            _ => {
                flush(&mut spans, current.take(), observations);
                current = Some(BuildingSpan {
                    side,
                    start_frame: frame,
                    end_frame: frame,
                    samples: 1,
                    wall_sign: offset.signum(),
                });
            }
        }
    }
    flush(&mut spans, current, observations);
    spans
}

fn flush(
    spans: &mut Vec<CornerSpan>,
    current: Option<BuildingSpan>,
    observations: &[SpatialObservation],
) {
    if let Some(span) = current {
        if span.samples >= CORNER_MIN_SAMPLES {
            spans.push(CornerSpan {
                side: span.side,
                start_frame: span.start_frame,
                end_frame: span.end_frame,
                ending: classify_ending(observations, &span),
            });
        }
    }
}

/// span の終わり方。終端直後の観測で、端側とそうでない側の左右が
/// 入れ替わったか、両者が壁から離れて中点の偏りが解けたかを確認する。
/// window 切れで直後の観測が無い(または確認に足りない)場合は
/// Unobserved に残し、終わり方を語らない。
fn classify_ending(observations: &[SpatialObservation], span: &BuildingSpan) -> CornerEnding {
    let mut swapped = 0usize;
    let mut separated = 0usize;
    for observation in observations.iter().filter(|observation| {
        observation.frame_index > span.end_frame
            && observation.frame_index <= span.end_frame.saturating_add(CORNER_END_LOOKAHEAD)
    }) {
        let Some((p1, p2)) = reliable_actor_pair(observation) else {
            continue;
        };
        let (cornered, other) = if span.side == 1 { (p1, p2) } else { (p2, p1) };
        // span 中は端側の人物が壁方向にいる。符号が反転していれば
        // 左右が入れ替わっている。
        if (cornered.anchor.x - other.anchor.x) * span.wall_sign < 0.0 {
            swapped += 1;
            continue;
        }
        // 左右そのままで中点の偏りが解けた = camera のクランプが外れる
        // ところまで壁から離れた。
        let midpoint = (p1.anchor.x + p2.anchor.x) / 2.0;
        if (midpoint - 0.5).abs() < CORNER_MIDPOINT_OFFSET {
            separated += 1;
        }
    }
    if swapped >= CORNER_MIN_SAMPLES {
        CornerEnding::SideSwap
    } else if separated >= CORNER_MIN_SAMPLES && swapped == 0 {
        CornerEnding::Separated
    } else {
        CornerEnding::Unobserved
    }
}

/// このフレームで端を背負っている側と、中点の偏り。確認できなければ None。
fn cornered_side(observation: &SpatialObservation) -> Option<(u8, f32)> {
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
    near_edge.then_some((side, offset))
}
