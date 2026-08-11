use super::super::*;
use super::support::*;

#[test]
fn synthetic_rows_cover_both_sides_and_strip_api() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 42, InputDir::DownLeft);
    paint_row_core(&mut rgba, "p2", 0, 7, InputDir::UpRight);

    let p1 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");
    let p2 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p2");
    assert_row_core(&p1[0], 42, InputDir::DownLeft);
    assert_row_core(&p2[0], 7, InputDir::UpRight);
    assert!(p1[1..].iter().all(|row| row.empty && !row.uncertain));
    assert!(p2[1..].iter().all(|row| row.empty && !row.uncertain));

    let strip = input_strip(&rgba);
    let p1_strip = read_input_row0_from_strip(&strip, TEST_W as u32, "p1");
    let p2_strip = read_input_row0_from_strip(&strip, TEST_W as u32, "p2");
    assert_row_core(&p1_strip, 42, InputDir::DownLeft);
    assert_row_core(&p2_strip, 7, InputDir::UpRight);
}

#[test]
fn synthetic_modern_badges_and_boxes() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 16, InputDir::Neutral);
    let y = ROW0_Y as usize;
    paint_plain_badge(&mut rgba, 122, y, [0, 255, 255]);
    paint_plain_badge(&mut rgba, 150, y, [255, 255, 0]);
    paint_plain_badge(&mut rgba, 178, y, [255, 0, 16]);
    paint_box_badge(&mut rgba, 206, y, [255, 96, 0]);
    paint_box_badge(&mut rgba, 240, y, [0, 255, 255]);
    paint_box_badge(&mut rgba, 274, y, [0, 96, 255]);

    let rows = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");
    assert_row_core(&rows[0], 16, InputDir::Neutral);
    assert_eq!(
        rows[0]
            .badges
            .iter()
            .map(|badge| badge.label())
            .collect::<Vec<_>>(),
        ["弱", "中", "強", "SP", "DI", "DP"]
    );
    assert!(!rows[0].auto);
    assert!(!rows[0].throw);
}

#[test]
fn a_p2_badge_far_from_the_direction_side_is_not_chained() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p2", 0, 8, InputDir::Neutral);
    paint_plain_badge(
        &mut rgba,
        P2_BADGE_X.0 as usize,
        ROW0_Y as usize,
        [0, 255, 255],
    );

    let rows = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p2");

    assert!(rows[0].badges.is_empty());
}

#[test]
fn synthetic_auto_and_throw_badges() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 6, InputDir::Right);
    paint_row_core(&mut rgba, "p2", 0, 17, InputDir::Left);
    let y = ROW0_Y as usize;
    paint_auto_badge(&mut rgba, 130, y);
    paint_throw_badge(&mut rgba, 1740, y);

    let p1 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");
    let p2 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p2");
    assert_row_core(&p1[0], 6, InputDir::Right);
    assert_row_core(&p2[0], 17, InputDir::Left);
    assert!(p1[0].auto);
    assert!(!p1[0].throw);
    assert!(!p2[0].auto);
    assert!(p2[0].throw);
}

#[test]
fn an_uncertain_direction_suppresses_auto_and_throw_marks() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 6, InputDir::Right);
    paint_row_core(&mut rgba, "p2", 0, 17, InputDir::Left);
    let row_y = ROW0_Y as usize;
    paint_auto_badge(&mut rgba, 130, row_y);
    paint_throw_badge(&mut rgba, 1740, row_y);

    let direction_y = (ROW0_Y as i32 + DIR_Y_OFF) as usize;
    for x0 in [P1_DIR_X as usize, P2_DIR_X as usize] {
        for y in direction_y..direction_y + DIR_H {
            for x in x0..x0 + DIR_W {
                let index = (y * TEST_W + x) * 4;
                rgba[index..index + 4].copy_from_slice(&[255; 4]);
            }
        }
    }

    let p1 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");
    let p2 = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p2");
    assert!(p1[0].uncertain);
    assert!(p2[0].uncertain);
    assert!(p1[0].badges.is_empty());
    assert!(p2[0].badges.is_empty());
    assert!(!p1[0].auto);
    assert!(!p1[0].throw);
    assert!(!p2[0].auto);
    assert!(!p2[0].throw);
}

#[test]
fn a_dim_direction_uses_the_low_threshold_read_without_becoming_uncertain() {
    let mut rgba = synthetic_frame();
    paint_row_core(&mut rgba, "p1", 0, 6, InputDir::Right);
    let direction_y = (ROW0_Y as i32 + DIR_Y_OFF) as usize;
    for y in direction_y..direction_y + DIR_H {
        for x in P1_DIR_X as usize..P1_DIR_X as usize + DIR_W {
            let index = (y * TEST_W + x) * 4;
            rgba[index..index + 4].copy_from_slice(&[128, 128, 128, 255]);
        }
    }
    let template = DIR_TEMPLATES[DIR_ORDER
        .iter()
        .position(|candidate| *candidate == InputDir::Right)
        .unwrap()];
    for (y, bits) in template.iter().enumerate() {
        for x in 0..DIR_W {
            if bits & (1 << x) != 0 {
                let index = ((direction_y + y) * TEST_W + P1_DIR_X as usize + x) * 4;
                rgba[index..index + 4].copy_from_slice(&[200, 200, 200, 255]);
            }
        }
    }

    let rows = read_input_rows(&rgba, TEST_W as u32, TEST_H as u32, "p1");
    assert_eq!(rows[0].dir, InputDir::Right);
    assert!(!rows[0].uncertain);
}
