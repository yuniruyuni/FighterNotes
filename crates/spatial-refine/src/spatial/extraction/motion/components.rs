use std::collections::VecDeque;

use super::super::grid::CellGrid;
use super::mask::{neighbors, MotionMask};
use super::MotionRegion;
use crate::spatial::SpatialRect;

pub(super) fn connected_regions(
    mask: &MotionMask,
    current: &CellGrid,
    source_width: u32,
    source_height: u32,
    cell_size: u32,
) -> Vec<MotionRegion> {
    let mut visited = vec![false; mask.connected.len()];
    let mut regions = Vec::new();
    for start in 0..mask.connected.len() {
        if !mask.connected[start] || visited[start] {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        let mut bounds = (mask.width, mask.height, 0usize, 0usize);
        let mut changed_cells = 0u32;
        let mut total_energy = 0u64;
        let mut effect_cells = 0u32;
        while let Some(index) = queue.pop_front() {
            let x = index % mask.width;
            let y = index / mask.width;
            bounds.0 = bounds.0.min(x);
            bounds.1 = bounds.1.min(y);
            bounds.2 = bounds.2.max(x);
            bounds.3 = bounds.3.max(y);
            if mask.energy[index] > 0 {
                changed_cells += 1;
                total_energy += mask.energy[index] as u64;
                let cell = current.cells[index];
                let high = cell.r.max(cell.g).max(cell.b);
                let low = cell.r.min(cell.g).min(cell.b);
                if high >= 145 && high.saturating_sub(low) >= 65 {
                    effect_cells += 1;
                }
            }
            for neighbor in neighbors(x, y, mask.width, mask.height) {
                if mask.connected[neighbor] && !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        if changed_cells == 0 {
            continue;
        }
        regions.push(MotionRegion {
            bounds: SpatialRect::new(
                bounds.0 as f32 * cell_size as f32 / source_width as f32,
                bounds.1 as f32 * cell_size as f32 / source_height as f32,
                ((bounds.2 + 1) as f32 * cell_size as f32 / source_width as f32).min(1.0),
                ((bounds.3 + 1) as f32 * cell_size as f32 / source_height as f32).min(1.0),
            ),
            changed_cells,
            energy: total_energy,
            effect_cells,
        });
    }
    regions
}

pub(super) fn merge_nearby(mut regions: Vec<MotionRegion>, merge_gap: f32) -> Vec<MotionRegion> {
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for a in 0..regions.len() {
            for b in (a + 1)..regions.len() {
                if should_merge(&regions[a], &regions[b], merge_gap) {
                    let other = regions.remove(b);
                    regions[a].merge(&other);
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    regions
}

fn should_merge(a: &MotionRegion, b: &MotionRegion, merge_gap: f32) -> bool {
    let horizontal_gap = if a.bounds.right < b.bounds.left {
        b.bounds.left - a.bounds.right
    } else if b.bounds.right < a.bounds.left {
        a.bounds.left - b.bounds.right
    } else {
        0.0
    };
    let vertical_overlap = a.bounds.bottom.min(b.bounds.bottom) - a.bounds.top.max(b.bounds.top);
    let min_height = a.bounds.height().min(b.bounds.height()).max(0.001);
    horizontal_gap <= merge_gap && vertical_overlap / min_height >= 0.20
}
