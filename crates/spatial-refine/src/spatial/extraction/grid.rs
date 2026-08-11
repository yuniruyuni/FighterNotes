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
        let mut cells = Vec::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_grid_averages_every_channel_and_partial_edge_cell() {
        let mut rgba = Vec::new();
        for y in 0..3u8 {
            for x in 0..3u8 {
                let red = x + y * 10;
                rgba.extend_from_slice(&[red, red + 50, red + 100, 255]);
            }
        }

        let grid = CellGrid::from_rgba(&rgba, 3, 3, 2);

        assert_eq!((grid.width, grid.height), (2, 2));
        assert_eq!(
            grid.cells
                .iter()
                .map(|cell| (cell.r, cell.g, cell.b))
                .collect::<Vec<_>>(),
            [(5, 55, 105), (7, 57, 107), (20, 70, 120), (22, 72, 122)]
        );
    }

    #[test]
    fn rgba_validation_checks_each_dimension_and_the_exact_buffer_length() {
        assert!(matches!(
            validate_rgba(&[0; 4], 0, 1),
            Err(SpatialError::InvalidDimensions)
        ));
        assert!(matches!(
            validate_rgba(&[0; 4], 1, 0),
            Err(SpatialError::InvalidDimensions)
        ));
        assert!(validate_rgba(&[0; 8], 2, 1).is_ok());
        assert!(matches!(
            validate_rgba(&[0; 7], 2, 1),
            Err(SpatialError::BufferTooSmall {
                expected: 8,
                actual: 7
            })
        ));
    }
}
