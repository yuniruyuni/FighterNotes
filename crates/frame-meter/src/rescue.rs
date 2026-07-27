use std::collections::HashMap;

use crate::color::{l2_dist, Bgr};
use crate::constants::{FAMILY_ASSIGN_DIST, RESCUE_MIN_FRAC};
use crate::model::CellState;
use crate::palette::PaletteName;

pub(crate) fn dominant_color_family(pixels: &[[u8; 3]]) -> Option<(CellState, Bgr)> {
    if pixels.is_empty() {
        return None;
    }
    let palette_colors: Vec<Bgr> = PaletteName::all().iter().map(|name| name.color()).collect();
    let mut nearest = vec![0usize; pixels.len()];
    let mut assigned = vec![false; pixels.len()];

    for (pixel_index, pixel) in pixels.iter().enumerate() {
        let bgr = [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32];
        let mut best_distance = f32::MAX;
        let mut best_index = 0;
        for (color_index, &color) in palette_colors.iter().enumerate() {
            let distance = l2_dist(bgr, color);
            if distance < best_distance {
                best_distance = distance;
                best_index = color_index;
            }
        }
        nearest[pixel_index] = best_index;
        assigned[pixel_index] = best_distance <= FAMILY_ASSIGN_DIST;
    }

    let mut family_counts: HashMap<CellState, usize> = HashMap::new();
    for (pixel_index, &palette_index) in nearest.iter().enumerate() {
        if !assigned[pixel_index] {
            continue;
        }
        if let Some(family) = PaletteName::all()[palette_index].state_family() {
            *family_counts.entry(family).or_insert(0) += 1;
        }
    }

    let (best_state, &best_count) = family_counts.iter().max_by_key(|(_, count)| *count)?;
    if (best_count as f32) < RESCUE_MIN_FRAC * pixels.len() as f32 {
        return None;
    }

    let mut sum = [0.0f32; 3];
    let mut count = 0usize;
    for (pixel_index, &palette_index) in nearest.iter().enumerate() {
        if !assigned[pixel_index] {
            continue;
        }
        if PaletteName::all()[palette_index]
            .state_family()
            .as_ref()
            .is_some_and(|family| family == best_state)
        {
            sum[0] += pixels[pixel_index][0] as f32;
            sum[1] += pixels[pixel_index][1] as f32;
            sum[2] += pixels[pixel_index][2] as f32;
            count += 1;
        }
    }

    Some((
        best_state.clone(),
        [
            sum[0] / count as f32,
            sum[1] / count as f32,
            sum[2] / count as f32,
        ],
    ))
}
