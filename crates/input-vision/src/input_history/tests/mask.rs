use super::super::mask::{dilate_rows, glyph_distance};

#[test]
fn dilation_expands_horizontally_and_vertically_without_wrapping() {
    let expanded = dilate_rows(&[0, 1 << 2, 0], 5);
    assert_eq!(expanded, [0b0_1110; 3]);

    let clipped = dilate_rows(&[1 << 4, 0, 0], 5);
    assert_eq!(clipped, [0b1_1000, 0b1_1000, 0]);

    let top_edge = dilate_rows(&[1, 0, 0], 5);
    assert_eq!(top_edge, [0b11, 0b11, 0]);

    let overlapping = dilate_rows(&[1 << 2, 1 << 3, 0], 5);
    assert_eq!(overlapping, [0b1_1110, 0b1_1110, 0b1_1100]);

    let horizontal_overlap = dilate_rows(&[0, (1 << 2) | (1 << 3), 0], 5);
    assert_eq!(horizontal_overlap, [0b1_1110; 3]);
}

#[test]
fn a_64_bit_mask_keeps_its_highest_column() {
    let expanded = dilate_rows(&[1 << 63], 64);
    assert_eq!(expanded[0], (1 << 63) | (1 << 62));
}

#[test]
fn glyph_distance_counts_missing_strokes_and_strokes_in_the_outline() {
    assert_eq!(glyph_distance(&[0], &[1 << 3], 8), 1);
    assert_eq!(glyph_distance(&[1 << 5], &[1 << 3], 8), 2);
    assert_eq!(glyph_distance(&[(1 << 3) | (1 << 7)], &[1 << 3], 8), 0);
}
