use crate::{
    extract_row_obs_from_strip, extract_row_obs_from_strip_with_digit_hint, CellState, CELL_COUNT,
    METER_STRIP_H,
};

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

#[test]
fn hinted_strip_extraction_scores_only_tracker_and_observed_windows() {
    let width = 1920;
    let mut strip = vec![0; width * METER_STRIP_H as usize * 4];
    fill_rows(&mut strip, width, 0, 38, [19, 201, 146, 255]);
    fill_rows(&mut strip, width, 40, 78, [176, 20, 93, 255]);

    let (full_left, _) = extract_row_obs_from_strip(&strip, width as u32, 1080);
    let (sparse_left, sparse_right) =
        extract_row_obs_from_strip_with_digit_hint(&strip, width as u32, 1080, Some((10, 12)));

    for index in (67..CELL_COUNT).chain(0..12) {
        assert_eq!(
            sparse_left.digit_correlation(index),
            full_left.digit_correlation(index)
        );
        assert!(sparse_right.digit_correlation(index).is_some());
    }
    assert!(sparse_left.digit_correlation(40).is_none());
    assert!(sparse_right.digit_correlation(40).is_none());
}

#[test]
fn hinted_strip_extraction_falls_back_to_all_digits_without_cursor_evidence() {
    let width = 1920;
    let strip = vec![0; width * METER_STRIP_H as usize * 4];

    let (left, right) =
        extract_row_obs_from_strip_with_digit_hint(&strip, width as u32, 1080, Some((10, 12)));

    for index in 0..CELL_COUNT {
        assert!(left.digit_correlation(index).is_some());
        assert!(right.digit_correlation(index).is_some());
    }
}
