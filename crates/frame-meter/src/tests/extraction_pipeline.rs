use crate::{extract_row_obs_from_strip, CellState, METER_STRIP_H};

fn fill_rows(pixels: &mut [u8], width: usize, y1: usize, y2: usize, rgba: [u8; 4]) {
    for row in y1..y2 {
        for column in 0..width {
            let index = (row * width + column) * 4;
            pixels[index..index + 4].copy_from_slice(&rgba);
        }
    }
}

#[test]
fn strip_extraction_uses_scaled_full_frame_origin() {
    let width = 1920;
    let scale = 2;
    let mut strip = vec![0; width * METER_STRIP_H as usize * scale * 4];
    fill_rows(&mut strip, width, 0, 76, [19, 201, 146, 255]);
    fill_rows(&mut strip, width, 80, 156, [176, 20, 93, 255]);

    let (left, right) = extract_row_obs_from_strip(&strip, width as u32, 2160);
    assert!(left.states.iter().all(|state| *state == CellState::Counter));
    assert!(right.states.iter().all(|state| *state == CellState::Active));
}
