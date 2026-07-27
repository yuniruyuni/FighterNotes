use super::super::*;

pub(super) const TEST_W: usize = 1920;
pub(super) const TEST_H: usize = 1080;

pub(super) fn synthetic_frame() -> Vec<u8> {
    let mut rgba = vec![128u8; TEST_W * TEST_H * 4];
    for alpha in rgba[3..].iter_mut().step_by(4) {
        *alpha = 255;
    }
    rgba
}

fn set_rgb(rgba: &mut [u8], x: usize, y: usize, rgb: [u8; 3]) {
    let index = (y * TEST_W + x) * 4;
    rgba[index..index + 3].copy_from_slice(&rgb);
    rgba[index + 3] = 255;
}

fn fill_rect(rgba: &mut [u8], x: usize, y: usize, width: usize, height: usize, rgb: [u8; 3]) {
    for py in y..y + height {
        for px in x..x + width {
            set_rgb(rgba, px, py, rgb);
        }
    }
}

fn paint_count(rgba: &mut [u8], side: &str, row: usize, count: u32) {
    let ones_x = (if side == "p1" { P1_ONES_X } else { P2_ONES_X }) as usize;
    let y0 = ROW0_Y as usize + ROW_PITCH as usize * row;
    for (position, digit) in count
        .to_string()
        .chars()
        .rev()
        .map(|value| value.to_digit(10).unwrap() as usize)
        .enumerate()
    {
        let x0 = ones_x - position * DIGIT_W;
        for y in 0..DIGIT_H {
            for x in 0..DIGIT_W {
                let value = DIGIT_NCC[digit].1[y][x];
                set_rgb(rgba, x0 + x, y0 + y, [value; 3]);
            }
        }
    }
}

fn paint_direction(rgba: &mut [u8], side: &str, row: usize, direction: InputDir) {
    let x0 = (if side == "p1" { P1_DIR_X } else { P2_DIR_X }) as usize;
    let y0 = (ROW0_Y as i32 + ROW_PITCH as i32 * row as i32 + DIR_Y_OFF) as usize;
    let template = DIR_TEMPLATES[DIR_ORDER
        .iter()
        .position(|candidate| *candidate == direction)
        .expect("synthetic direction must have a template")];
    for (y, bits) in template.iter().enumerate() {
        for x in 0..DIR_W {
            if bits & (1 << x) != 0 {
                set_rgb(rgba, x0 + x, y0 + y, [255; 3]);
            }
        }
    }
}

pub(super) fn paint_row_core(
    rgba: &mut [u8],
    side: &str,
    row: usize,
    count: u32,
    direction: InputDir,
) {
    paint_count(rgba, side, row, count);
    paint_direction(rgba, side, row, direction);
}

pub(super) fn input_strip(rgba: &[u8]) -> Vec<u8> {
    let y1 = INPUT_STRIP_Y as usize;
    let y2 = y1 + INPUT_STRIP_H as usize;
    let mut strip = Vec::with_capacity(TEST_W * (y2 - y1) * 4);
    for y in y1..y2 {
        let start = y * TEST_W * 4;
        strip.extend_from_slice(&rgba[start..start + TEST_W * 4]);
    }
    strip
}

pub(super) fn assert_row_core(row: &InputRow, count: u32, direction: InputDir) {
    assert_eq!(row.count, Some(count));
    assert_eq!(row.dir, direction);
    assert!(!row.empty);
    assert!(!row.uncertain);
}

pub(super) fn paint_plain_badge(rgba: &mut [u8], x: usize, y: usize, rgb: [u8; 3]) {
    fill_rect(rgba, x, y, 22, DIGIT_H, [24; 3]);
    fill_rect(rgba, x, y + 4, 22, 10, rgb);
}

pub(super) fn paint_box_badge(rgba: &mut [u8], x: usize, y: usize, rgb: [u8; 3]) {
    fill_rect(rgba, x, y, 28, DIGIT_H, [24; 3]);
    fill_rect(rgba, x, y + 4, 28, 10, rgb);
    fill_rect(rgba, x + 8, y + 7, 10, 3, [240; 3]);
}

pub(super) fn paint_auto_badge(rgba: &mut [u8], x: usize, y: usize) {
    fill_rect(rgba, x, y + 3, 28, 12, [24; 3]);
    fill_rect(rgba, x + 8, y + 7, 10, 3, [240; 3]);
}

pub(super) fn paint_throw_badge(rgba: &mut [u8], x: usize, y: usize) {
    fill_rect(rgba, x, y, 16, 2, [24; 3]);
    fill_rect(rgba, x, y + 14, 16, 2, [24; 3]);
    fill_rect(rgba, x, y + 5, 16, 6, [240; 3]);
}
