//! Per-frame camera motion estimation from background strips.
//!
//! SF6's replay camera pans with the players' midpoint and zooms with their
//! separation, so normalized screen distance is not proportional to game
//! distance. The stage itself is world-anchored: horizontal luma strips from
//! the backdrop and the floor shift by the camera pan and stretch by the
//! zoom. Correlating each strip against the previous frame therefore
//! recovers the camera, without knowing anything about the stage.
//!
//! Pixels near the tracked actors are masked out so body motion does not
//! contaminate the estimate; hitstop VFX is compact and rarely touches all
//! strips at once.

use super::super::{ActorObservation, CameraMotion};

/// Luma strips sampled from fixed rows, kept between frames.
pub(super) struct CameraStrips {
    rows: Vec<Vec<f32>>,
    width: usize,
}

/// Normalized rows sampled for correlation. Two backdrop rows above the
/// fighters plus two floor rows below the frame-meter overlay band.
const STRIP_ROWS: [f32; 4] = [0.18, 0.30, 0.93, 0.955];
/// Maximum per-frame shift searched, in pixels.
const MAX_SHIFT: i32 = 6;
/// Segment centers, as fractions of the frame width. Four narrow segments
/// survive actor masks better than two wide ones, and any two with enough
/// spread still give a zoom baseline.
const SEGMENT_CENTER_FRACTIONS: [f32; 4] = [0.12, 0.38, 0.62, 0.88];
/// Half-width of each correlated segment, as a fraction of the frame width.
/// 隣り合う 2 列だけが残っても間隔は 0.24W あり、pan と zoom を分離できる。
const SEGMENT_HALF_WIDTH: f32 = 0.11;
/// Half-width of the mask placed over each tracked actor.
const ACTOR_MASK_HALF_WIDTH: f32 = 0.16;

/// The actor bodies are the moving foreground; their columns must not vote
/// on camera motion. A hitstop spark appears between the bodies with little
/// slack, so the two actor masks already cover it.
pub(super) fn masked_centers(
    p1: Option<&ActorObservation>,
    p2: Option<&ActorObservation>,
) -> Vec<f32> {
    [p1, p2]
        .into_iter()
        .flatten()
        .map(|actor| actor.anchor.x)
        .collect()
}

pub(super) fn sample_strips(rgba: &[u8], width: u32, height: u32) -> CameraStrips {
    let rows = STRIP_ROWS
        .iter()
        .map(|&normalized| {
            let y = ((normalized * height as f32) as usize).min(height as usize - 1);
            (0..width as usize)
                .map(|x| {
                    let index = (y * width as usize + x) * 4;
                    rgba[index] as f32 + rgba[index + 1] as f32 + rgba[index + 2] as f32
                })
                .collect()
        })
        .collect();
    CameraStrips {
        rows,
        width: width as usize,
    }
}

/// Estimate pan (in normalized screen x) and zoom ratio between the previous
/// and current strips. `masked_centers_x` are normalized x positions to
/// blank out (tracked actors, contact spark).
pub(super) fn estimate(
    previous: &CameraStrips,
    current: &CameraStrips,
    masked_centers_x: &[f32],
) -> Option<CameraMotion> {
    if previous.width != current.width || previous.width == 0 {
        return None;
    }
    let width = current.width;
    let mask = build_mask(width, masked_centers_x);

    // セグメント列ごとに strip 横断の median shift を取り、(x, shift) の
    // 標本にする。shift は x に対して線形(pan + zoom 伸縮)のはず。
    let mut columns: Vec<(f32, f32)> = Vec::new();
    let mut usable_segments = 0usize;
    for &fraction in &SEGMENT_CENTER_FRACTIONS {
        let center = (fraction * width as f32) as usize;
        let half = (SEGMENT_HALF_WIDTH * width as f32) as usize;
        let start = center.saturating_sub(half);
        let end = (center + half).min(width);
        let mut shifts: Vec<f32> = previous
            .rows
            .iter()
            .zip(&current.rows)
            .filter_map(|(previous_row, current_row)| {
                correlate(previous_row, current_row, &mask, start, end)
            })
            .collect();
        if !shifts.is_empty() {
            usable_segments += shifts.len();
            columns.push((center as f32, median(&mut shifts)));
        }
    }
    if columns.len() < 2 {
        return None;
    }

    // shift(x) = pan + b * (x - W/2) の最小二乗。b がズームによる伸縮率。
    // 列は少なくとも 2 つの異なる中心を持つので分母は正になる。
    let count = columns.len() as f32;
    let sum_x: f32 = columns.iter().map(|(x, _)| x).sum();
    let sum_shift: f32 = columns.iter().map(|(_, shift)| shift).sum();
    let sum_xx: f32 = columns.iter().map(|(x, _)| x * x).sum();
    let sum_xshift: f32 = columns.iter().map(|(x, shift)| x * shift).sum();
    let stretch = (count * sum_xshift - sum_x * sum_shift) / (count * sum_xx - sum_x * sum_x);
    let pan_center = (sum_shift - stretch * sum_x) / count + stretch * (width as f32 / 2.0);
    let zoom_ratio = 1.0 + stretch;
    let confidence = (0.30 + 0.06 * usable_segments as f32).min(0.90);
    Some(CameraMotion {
        pan_dx: pan_center / width as f32,
        zoom_ratio,
        confidence,
    })
}

fn build_mask(width: usize, masked_centers_x: &[f32]) -> Vec<bool> {
    let mut mask = vec![false; width];
    for &center in masked_centers_x {
        let half = (ACTOR_MASK_HALF_WIDTH * width as f32) as i32;
        let center = (center * width as f32) as i32;
        for x in (center - half).max(0)..((center + half).min(width as i32 - 1) + 1) {
            mask[x as usize] = true;
        }
    }
    mask
}

/// Sub-pixel 1D correlation of one segment: best integer shift by masked SAD
/// plus parabolic refinement around it. Returns the shift of the background
/// from previous to current (positive = content moved right).
fn correlate(
    previous: &[f32],
    current: &[f32],
    mask: &[bool],
    start: usize,
    end: usize,
) -> Option<f32> {
    let mut costs = [0.0f32; (2 * MAX_SHIFT + 1) as usize];
    let mut counts = [0u32; (2 * MAX_SHIFT + 1) as usize];
    for (slot, cost) in costs.iter_mut().enumerate() {
        let shift = slot as i32 - MAX_SHIFT;
        let mut count = 0u32;
        for x in start..end {
            let source = x as i32 - shift;
            if source < 0 || source as usize >= previous.len() {
                continue;
            }
            if mask[x] || mask[source as usize] {
                continue;
            }
            *cost += (current[x] - previous[source as usize]).abs();
            count += 1;
        }
        counts[slot] = count;
        if count > 0 {
            *cost /= count as f32;
        } else {
            *cost = f32::INFINITY;
        }
    }
    let minimum_samples = ((end - start) / 4) as u32;
    let best = costs
        .iter()
        .enumerate()
        .filter(|(slot, _)| counts[*slot] >= minimum_samples.max(8))
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|(slot, _)| slot)?;
    if !costs[best].is_finite() {
        return None;
    }
    let shift = best as i32 - MAX_SHIFT;
    // 端で最小になった場合は探索範囲不足なので採用しない。
    if best == 0 || best == costs.len() - 1 {
        return None;
    }
    let (a, b, c) = (costs[best - 1], costs[best], costs[best + 1]);
    let denominator = a - 2.0 * b + c;
    let refinement = if denominator.abs() > f32::EPSILON {
        (0.5 * (a - c) / denominator).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    Some(shift as f32 + refinement)
}

fn median(values: &mut [f32]) -> f32 {
    values.sort_by(|a, b| a.total_cmp(b));
    values[values.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::super::super::{SpatialPoint, SpatialRect};
    use super::*;

    fn actor(x: f32) -> ActorObservation {
        ActorObservation {
            anchor: SpatialPoint::new(x, 0.9),
            bounds: SpatialRect::new(x - 0.05, 0.6, x + 0.05, 0.9),
            confidence: 0.72,
            observed: true,
            ground_anchor: true,
            discontinuity: false,
        }
    }

    fn strips_from(width: usize, luma: impl Fn(f32) -> f32) -> CameraStrips {
        CameraStrips {
            rows: (0..4)
                .map(|_| (0..width).map(|x| luma(x as f32)).collect())
                .collect(),
            width,
        }
    }

    fn wave(x: f32) -> f32 {
        100.0 + 80.0 * (x * 0.37).sin()
    }

    #[test]
    fn masked_centers_lists_the_tracked_bodies() {
        let p1 = actor(0.3);
        let p2 = actor(0.7);
        assert_eq!(masked_centers(Some(&p1), Some(&p2)), vec![0.3, 0.7]);
        assert_eq!(masked_centers(None, Some(&p2)), vec![0.7]);
        assert_eq!(masked_centers(Some(&p1), None), vec![0.3]);
        assert!(masked_centers(None, None).is_empty());
    }

    #[test]
    fn sample_strips_reads_the_configured_rows() {
        // 画素値 = 行番号のフレーム。各 strip はその行の r+g+b になる。
        let width = 8u32;
        let height = 100u32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let index = (y * width as usize + x) * 4;
                rgba[index] = y as u8;
                rgba[index + 1] = y as u8;
                rgba[index + 2] = y as u8;
            }
        }
        let strips = sample_strips(&rgba, width, height);
        let expected_rows = [18.0, 30.0, 93.0, 95.0];
        for (row, expected) in strips.rows.iter().zip(expected_rows) {
            assert!(
                row.iter().all(|&value| value == expected * 3.0),
                "{expected}"
            );
        }
    }

    #[test]
    fn estimate_recovers_a_pure_pan() {
        let previous = strips_from(480, wave);
        let current = strips_from(480, |x| wave(x - 2.0));
        let motion = estimate(&previous, &current, &[]).expect("camera motion");
        assert!((motion.pan_dx * 480.0 - 2.0).abs() < 0.15, "{motion:?}");
        assert!((motion.zoom_ratio - 1.0).abs() < 0.002, "{motion:?}");
        // 4 strip x 4 セグメントすべてが使えたときの confidence は上限。
        assert_eq!(motion.confidence, 0.90);
    }

    #[test]
    fn estimate_recovers_a_zoom_stretch() {
        let previous = strips_from(480, wave);
        // 中心を固定して 1.02 倍へ引き伸ばした背景。
        let current = strips_from(480, |x| wave(240.0 + (x - 240.0) / 1.02));
        let motion = estimate(&previous, &current, &[]).expect("camera motion");
        assert!((motion.zoom_ratio - 1.0196).abs() < 0.004, "{motion:?}");
        assert!((motion.pan_dx * 480.0).abs() < 0.3, "{motion:?}");
    }

    #[test]
    fn masked_foreground_does_not_vote() {
        let previous = strips_from(480, wave);
        // セグメント 2 (130..234px) を覆う帯だけが 5px 動く。
        let band = 120.0..244.0;
        let current = strips_from(480, |x| {
            if band.contains(&x) {
                wave(x - 5.0)
            } else {
                wave(x)
            }
        });
        // マスク無しでは前景の動きが fit を引っ張る。
        let polluted = estimate(&previous, &current, &[]).expect("camera motion");
        assert!((polluted.pan_dx * 480.0).abs() > 0.5, "{polluted:?}");
        // 帯の中心をマスクすれば静止背景だけが残る。
        let masked = estimate(&previous, &current, &[182.0 / 480.0]).expect("camera motion");
        assert!((masked.pan_dx * 480.0).abs() < 0.3, "{masked:?}");
        assert!((masked.zoom_ratio - 1.0).abs() < 0.002, "{masked:?}");
        // 使えたセグメントは 4 strip x 3 列で、confidence はまだ上限に届く
        // 手前ではない。2 列マスクした場合の値も式どおりに落ちる。
        let two_masked = estimate(&previous, &current, &[0.12, 0.38]).expect("camera motion");
        assert!(
            (two_masked.confidence - 0.78).abs() < 1e-6,
            "{two_masked:?}"
        );
    }

    /// 列が非対称に残った場合の最小二乗の値を式どおりに固定する。
    /// 列 = (182, 5), (297, 0), (422, 0) → pan 2.9055px, zoom 0.97947。
    #[test]
    fn least_squares_fit_matches_the_asymmetric_column_solution() {
        let previous = strips_from(480, wave);
        let band = 120.0..244.0;
        let current = strips_from(480, |x| {
            if band.contains(&x) {
                wave(x - 5.0)
            } else {
                wave(x)
            }
        });
        // 左端のセグメントだけをマスクして列を 3 本に減らす。
        let motion = estimate(&previous, &current, &[57.0 / 480.0]).expect("camera motion");
        assert!((motion.pan_dx * 480.0 - 2.9055).abs() < 0.15, "{motion:?}");
        assert!((motion.zoom_ratio - 0.97947).abs() < 0.002, "{motion:?}");
    }

    #[test]
    fn build_mask_covers_the_exact_actor_interval() {
        let mask = build_mask(480, &[0.5]);
        // 240 ± 76 → 164..=316 を覆う。
        assert!(!mask[163]);
        assert!(mask[164]);
        assert!(mask[316]);
        assert!(!mask[317]);
        // 端では画面の内側へ切り詰める。
        let left_edge = build_mask(480, &[0.0]);
        assert!(left_edge[0] && left_edge[76] && !left_edge[77]);
        let right_edge = build_mask(480, &[1.0]);
        assert!(right_edge[479] && right_edge[404] && !right_edge[403]);
    }

    #[test]
    fn median_sorts_and_takes_the_upper_middle() {
        assert_eq!(median(&mut [4.0, 1.0, 3.0, 9.0]), 4.0);
        assert_eq!(median(&mut [5.0]), 5.0);
    }

    #[test]
    fn estimate_refuses_thin_or_mismatched_evidence() {
        let previous = strips_from(480, wave);
        let current = strips_from(480, wave);
        // 全セグメントをマスクすると列が足りない。
        assert!(estimate(&previous, &current, &[0.12, 0.38, 0.62, 0.88]).is_none());
        // 幅が変わった strip とは比較しない。
        let narrow = strips_from(320, wave);
        assert!(estimate(&previous, &narrow, &[]).is_none());
        // 探索範囲を超える移動は端で最小になるため採用しない。
        let runaway = strips_from(480, |x| wave(x - 9.0));
        assert!(estimate(&previous, &runaway, &[]).is_none());
    }
}
