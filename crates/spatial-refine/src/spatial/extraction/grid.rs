use super::super::SpatialError;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CellColor {
    pub(super) r: u8,
    pub(super) g: u8,
    pub(super) b: u8,
}

pub(super) struct CellGrid {
    pub(super) cells: Vec<CellColor>,
    pub(super) width: usize,
    pub(super) height: usize,
}

impl CellGrid {
    pub(super) fn from_rgba(rgba: &[u8], width: u32, height: u32, cell_size: u32) -> Self {
        let grid_width = width.div_ceil(cell_size) as usize;
        let grid_height = height.div_ceil(cell_size) as usize;
        let mut cells = Vec::with_capacity(grid_width * grid_height);
        for grid_y in 0..grid_height {
            let y0 = grid_y as u32 * cell_size;
            let y1 = (y0 + cell_size).min(height);
            for grid_x in 0..grid_width {
                let x0 = grid_x as u32 * cell_size;
                let x1 = (x0 + cell_size).min(width);
                let mut channels = [0u64; 3];
                let mut count = 0u64;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let index = (y as usize * width as usize + x as usize) * 4;
                        channels[0] += rgba[index] as u64;
                        channels[1] += rgba[index + 1] as u64;
                        channels[2] += rgba[index + 2] as u64;
                        count += 1;
                    }
                }
                cells.push(CellColor {
                    r: (channels[0] / count) as u8,
                    g: (channels[1] / count) as u8,
                    b: (channels[2] / count) as u8,
                });
            }
        }
        Self {
            cells,
            width: grid_width,
            height: grid_height,
        }
    }
}

pub(super) fn validate_rgba(rgba: &[u8], width: u32, height: u32) -> Result<(), SpatialError> {
    if width == 0 || height == 0 {
        return Err(SpatialError::InvalidDimensions);
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(SpatialError::InvalidDimensions)?;
    if rgba.len() < expected {
        return Err(SpatialError::BufferTooSmall {
            expected,
            actual: rgba.len(),
        });
    }
    Ok(())
}
