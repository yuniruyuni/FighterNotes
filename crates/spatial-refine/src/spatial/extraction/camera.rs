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

use super::super::CameraMotion;

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
const SEGMENT_HALF_WIDTH: f32 = 0.11;
/// Minimum horizontal spread between usable segment columns, as a fraction
/// of the frame width. Below this, pan and zoom cannot be separated.
const MIN_COLUMN_SPREAD: f32 = 0.20;
/// Half-width of the mask placed over each tracked actor.
const ACTOR_MASK_HALF_WIDTH: f32 = 0.16;

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
    let spread = columns.last().unwrap().0 - columns.first().unwrap().0;
    if spread < MIN_COLUMN_SPREAD * width as f32 {
        return None;
    }

    // shift(x) = pan + b * (x - W/2) の最小二乗。b がズームによる伸縮率。
    let count = columns.len() as f32;
    let mean_x = columns.iter().map(|(x, _)| x).sum::<f32>() / count;
    let mean_shift = columns.iter().map(|(_, s)| s).sum::<f32>() / count;
    let mut covariance = 0.0;
    let mut variance = 0.0;
    for (x, shift) in &columns {
        covariance += (x - mean_x) * (shift - mean_shift);
        variance += (x - mean_x) * (x - mean_x);
    }
    let stretch = covariance / variance;
    let pan_center = mean_shift + stretch * (width as f32 / 2.0 - mean_x);
    let zoom_ratio = 1.0 + stretch;
    if !zoom_ratio.is_finite() || !pan_center.is_finite() {
        return None;
    }
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
