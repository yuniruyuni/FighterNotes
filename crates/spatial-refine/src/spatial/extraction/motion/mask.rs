use super::super::super::{SpatialConfig, SpatialPoint};
use super::super::grid::CellGrid;

pub(super) struct MotionMask {
    pub(super) energy: Vec<u16>,
    pub(super) connected: Vec<bool>,
    pub(super) width: usize,
    pub(super) height: usize,
}

pub(super) fn motion_mask(
    previous: &CellGrid,
    current: &CellGrid,
    config: &SpatialConfig,
) -> MotionMask {
    debug_assert_eq!(
        (previous.width, previous.height),
        (current.width, current.height)
    );
    let mut energy = vec![0u16; current.cells.len()];
    for y in 0..current.height {
        for x in 0..current.width {
            let index = y * current.width + x;
            let point = SpatialPoint::new(
                (x as f32 + 0.5) / current.width as f32,
                (y as f32 + 0.5) / current.height as f32,
            );
            if !config.playfield.contains(point)
                || config
                    .excluded_regions
                    .iter()
                    .any(|region| region.contains(point))
            {
                continue;
            }
            let a = previous.cells[index];
            let b = current.cells[index];
            let delta =
                a.r.abs_diff(b.r)
                    .max(a.g.abs_diff(b.g))
                    .max(a.b.abs_diff(b.b));
            if delta >= config.motion_threshold {
                energy[index] = delta as u16;
            }
        }
    }

    let mut cleaned = vec![false; energy.len()];
    for y in 0..current.height {
        for x in 0..current.width {
            let index = y * current.width + x;
            if energy[index] == 0 {
                continue;
            }
            let active_neighbors = neighbors(x, y, current.width, current.height)
                .filter(|&neighbor| energy[neighbor] > 0)
                .count() as u8;
            if active_neighbors >= config.min_motion_neighbors {
                cleaned[index] = true;
            }
        }
    }

    // One-cell dilation connects compression-fragmented silhouettes while the
    // component's changed-cell count still comes from the undilated mask.
    let mut connected = cleaned.clone();
    for y in 0..current.height {
        for x in 0..current.width {
            let index = y * current.width + x;
            if cleaned[index] {
                for neighbor in neighbors_including_self(x, y, current.width, current.height) {
                    connected[neighbor] = true;
                }
            }
        }
    }
    MotionMask {
        energy,
        connected,
        width: current.width,
        height: current.height,
    }
}

pub(super) fn neighbors(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = usize> {
    let x0 = x.saturating_sub(1);
    let y0 = y.saturating_sub(1);
    let x1 = (x + 1).min(width - 1);
    let y1 = (y + 1).min(height - 1);
    (y0..=y1).flat_map(move |ny| {
        (x0..=x1).filter_map(move |nx| (nx != x || ny != y).then_some(ny * width + nx))
    })
}

fn neighbors_including_self(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = usize> {
    let x0 = x.saturating_sub(1);
    let y0 = y.saturating_sub(1);
    let x1 = (x + 1).min(width - 1);
    let y1 = (y + 1).min(height - 1);
    (y0..=y1).flat_map(move |ny| (x0..=x1).map(move |nx| ny * width + nx))
}
