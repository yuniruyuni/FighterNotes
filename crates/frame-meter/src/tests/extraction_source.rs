use crate::extraction::source::RowSource;

fn solid_rgba(width: usize, height: usize, rgba: [u8; 4]) -> Vec<u8> {
    let mut pixels = vec![0; width * height * 4];
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&rgba);
    }
    pixels
}

#[test]
fn row_source_scales_geometry_channels_and_reference_rows() {
    let rgba = solid_rgba(192, 108, [10, 20, 30, 255]);
    let row = RowSource::new(&rgba, 192, 108, 0)
        .read_row(796, 834, &[0, 13, 26], &[6, 20])
        .unwrap();

    assert_eq!(
        (row.width, row.height, row.trim_y, row.patch_height),
        (120, 4, 1, 2)
    );
    assert_eq!(row.region1_rows, [0, 1]);
    assert_eq!(row.region2_rows, [0, 1]);
    assert!(row.bgr.iter().all(|pixel| *pixel == [30, 20, 10]));
    assert!(row.value.iter().all(|value| *value == 30.0));
    assert!(row
        .saturation
        .iter()
        .all(|value| (*value - 170.0).abs() < 1e-4));
}

#[test]
fn row_source_skips_negative_strip_rows_but_reads_strip_row_zero() {
    let rgba = solid_rgba(192, 3, [10, 20, 30, 255]);
    let row = RowSource::new(&rgba, 192, 108, 80)
        .read_row(796, 834, &[], &[])
        .unwrap();

    assert!(row.bgr[..row.width].iter().all(|pixel| *pixel == [0; 3]));
    assert!(row.bgr[row.width..2 * row.width]
        .iter()
        .all(|pixel| *pixel == [30, 20, 10]));
}

#[test]
fn row_source_rejects_either_zero_dimension() {
    let rgba = vec![0; 192 * 108 * 4];
    assert!(RowSource::new(&[], 0, 108, 0)
        .read_row(796, 834, &[], &[])
        .is_none());
    assert!(RowSource::new(&[], 192, 0, 0)
        .read_row(796, 834, &[], &[])
        .is_none());
    assert!(RowSource::new(&rgba, 1, 108, 0)
        .read_row(796, 834, &[], &[])
        .is_none());
    assert!(RowSource::new(&rgba, 192, 108, 0)
        .read_row(800, 800, &[], &[])
        .is_none());
}

#[test]
fn row_source_does_not_read_an_incomplete_rgba_pixel() {
    let first_source_index = (79 * 192 + 35) * 4;
    let rgba = vec![255; first_source_index + 3];
    let row = RowSource::new(&rgba, 192, 108, 0)
        .read_row(796, 834, &[], &[])
        .unwrap();

    assert!(row.bgr.iter().all(|pixel| *pixel == [0; 3]));
}

#[test]
fn row_source_uses_the_exact_global_column_and_rgba_channels() {
    let width = 192usize;
    let height = 108usize;
    let mut rgba = vec![0; width * height * 4];
    for row in 0..height {
        for column in 0..width {
            let index = (row * width + column) * 4;
            rgba[index..index + 4].copy_from_slice(&[
                column as u8,
                row as u8,
                (column ^ row) as u8,
                255,
            ]);
        }
    }
    let row = RowSource::new(&rgba, width as u32, height as u32, 0)
        .read_row(796, 834, &[], &[])
        .unwrap();

    assert_eq!(row.bgr[0], [35 ^ 79, 79, 35]);
    assert_eq!(row.bgr[1], [36 ^ 79, 79, 36]);
}

#[test]
fn reference_rows_scale_round_deduplicate_and_clamp() {
    let width = 480;
    let height = 270;
    let rgba = solid_rgba(width, height, [10, 20, 30, 255]);
    let row = RowSource::new(&rgba, width as u32, height as u32, 0)
        .read_row(796, 834, &[0, 6, 13, 20, 26], &[])
        .unwrap();

    assert_eq!(row.patch_height, 7);
    assert_eq!(row.region1_rows, [0, 2, 4, 5, 6]);
}
