use super::super::*;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 180;

pub fn blank_frame() -> Vec<u8> {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[32, 36, 40, 255]);
    }
    rgba
}

pub fn rect(frame: &mut [u8], x: u32, y: u32, width: u32, height: u32, color: [u8; 3]) {
    for py in y..(y + height).min(HEIGHT) {
        for px in x..(x + width).min(WIDTH) {
            let index = (py as usize * WIDTH as usize + px as usize) * 4;
            frame[index..index + 3].copy_from_slice(&color);
        }
    }
}

pub fn test_config() -> SpatialConfig {
    SpatialConfig {
        cell_size: 4,
        motion_threshold: 12,
        min_motion_neighbors: 1,
        playfield: SpatialRect::new(0.0, 0.0, 1.0, 1.0),
        excluded_regions: Vec::new(),
        actor_min_changed_cells: 10,
        actor_min_height: 0.08,
        region_merge_gap: 0.08,
        projectile_min_changed_cells: 1,
        projectile_max_changed_cells: 120,
        projectile_max_width: 0.18,
        projectile_max_height: 0.18,
        actor_exclusion_dx: 0.10,
        ..SpatialConfig::default()
    }
}

pub fn hints() -> SpatialHints {
    SpatialHints {
        p1: ActorHint {
            anchor: Some(SpatialPoint::new(0.25, 0.84)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        p2: ActorHint {
            anchor: Some(SpatialPoint::new(0.78, 0.84)),
            allow_discontinuity: false,
            allow_airborne: false,
        },
        contact_effect: false,
        sides_certain: false,
    }
}
