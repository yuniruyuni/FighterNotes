use super::super::*;
use super::support::{paint_row_core, synthetic_frame, TEST_H, TEST_W};

#[test]
fn invalid_geometry_and_short_strip_return_empty_results() {
    assert!(read_input_rows(&[], 0, 0, "p1").is_empty());

    let row = read_input_row0_from_strip(&[], TEST_W as u32, "p2");
    assert!(row.empty);
    assert!(!row.uncertain);
    assert_eq!(row.count, None);
    assert_eq!(row.dir, InputDir::Unknown);
    assert!(row.badges.is_empty());
}

#[test]
fn low_information_frame_does_not_invent_inputs() {
    let rgba = vec![0; TEST_W * TEST_H * 4];
    let rows = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");

    assert_eq!(rows.len(), INPUT_ROWS);
    assert!(rows.iter().all(|row| row.empty && !row.uncertain));
    assert!(rows.iter().all(|row| {
        row.count.is_none()
            && row.dir == InputDir::Unknown
            && row.badges.is_empty()
            && !row.auto
            && !row.throw
    }));
}

#[test]
fn saturated_stage_color_is_not_a_dark_badge_rim() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 1, InputDir::Neutral);
    for y in ROW0_Y as usize..ROW0_Y as usize + DIGIT_H {
        for x in P1_BADGE_X.0 as usize..P1_BADGE_X.0 as usize + 24 {
            let index = (y * TEST_W + x) * 4;
            rgba[index..index + 3].copy_from_slice(&[210, 18, 30]);
        }
    }

    let rows = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");

    assert!(rows[0].badges.is_empty());
}
