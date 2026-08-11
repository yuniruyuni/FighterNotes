use crate::extraction::metrics::{
    cell_bounds, mean_value, min_cell_patch_width, white_row_fraction, write_column_means,
    write_digit_patch, CellBounds,
};
use crate::extraction::source::RowPixels;
use crate::{DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W};

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

fn make_pixels(width: usize, height: usize, trim_y: usize) -> RowPixels {
    let size = width * height;
    RowPixels {
        width,
        height,
        trim_y,
        patch_height: height - 2 * trim_y,
        region1_rows: vec![],
        region2_rows: vec![],
        bgr: vec![[0; 3]; size],
        value: (0..size).map(|value| value as f32).collect(),
        saturation: vec![0.0; size],
    }
}

#[test]
fn cell_geometry_preserves_trimmed_bounds_and_minimum_width() {
    let first = cell_bounds(1200, 0).unwrap();
    let last = cell_bounds(1200, 79).unwrap();
    assert_eq!((first.x1, first.x2, first.width()), (1, 14, 13));
    assert_eq!((last.x1, last.x2, last.width()), (1186, 1199, 13));
    assert_eq!(cell_bounds(1200, 80).map(CellBounds::width), None);
    assert_eq!(cell_bounds(2, 39).map(CellBounds::width), Some(1));
    assert_eq!(cell_bounds(1, 79).map(CellBounds::width), None);
    assert_eq!(min_cell_patch_width(1200), 13);
    assert_eq!(min_cell_patch_width(80), 0);
}

#[test]
fn value_metrics_use_only_patch_rows_and_requested_columns() {
    let pixels = make_pixels(4, 4, 1);
    let bounds = CellBounds { x1: 1, x2: 3 };

    assert_close(mean_value(&pixels, bounds), 7.5);
    assert_eq!(mean_value(&pixels, CellBounds { x1: 2, x2: 2 }), 0.0);

    let mut means = [0.0; 2];
    write_column_means(&mut means, &pixels, bounds);
    assert_eq!(means, [7.0, 8.0]);
}

#[test]
fn digit_patch_requires_both_dimensions_and_normalizes_values() {
    let width = DIGIT_TEMPLATE_W;
    let height = DIGIT_TEMPLATE_H;
    let pixels = make_pixels(width, height, 0);
    let bounds = CellBounds { x1: 0, x2: width };
    let mut output = vec![0.0; width * height];

    assert!(write_digit_patch(&mut output, &pixels, bounds));
    assert_close(output.iter().sum::<f32>() / output.len() as f32, 0.0);
    assert_close(
        output.iter().map(|value| value * value).sum::<f32>() / output.len() as f32,
        1.0,
    );
    assert!(output.first().unwrap() < &-1.0);
    assert!(output.last().unwrap() > &1.0);

    let short = make_pixels(width, height - 1, 0);
    assert!(!write_digit_patch(&mut output, &short, bounds));
    let narrow = CellBounds {
        x1: 0,
        x2: width - 1,
    };
    assert!(!write_digit_patch(&mut output, &pixels, narrow));
}

#[test]
fn digit_patch_reads_the_requested_offset_from_each_physical_row() {
    let width = DIGIT_TEMPLATE_W + 4;
    let height = DIGIT_TEMPLATE_H + 2;
    let pixels = make_pixels(width, height, 1);
    let bounds = CellBounds {
        x1: 2,
        x2: 2 + DIGIT_TEMPLATE_W,
    };
    let mut actual = vec![0.0; DIGIT_TEMPLATE_W * DIGIT_TEMPLATE_H];
    assert!(write_digit_patch(&mut actual, &pixels, bounds));

    let mut raw = Vec::new();
    for row in 0..DIGIT_TEMPLATE_H {
        for column in 0..DIGIT_TEMPLATE_W {
            raw.push(pixels.value[(row + 1) * width + 2 + column]);
        }
    }
    let mean = raw.iter().sum::<f32>() / raw.len() as f32;
    let variance = raw
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum::<f32>()
        / raw.len() as f32;
    let denominator = variance.sqrt().max(1.0);
    for (index, value) in actual.iter().enumerate() {
        assert_close(*value, (raw[index] - mean) / denominator);
    }
}

#[test]
fn white_fraction_requires_high_value_and_low_saturation_per_row() {
    let mut pixels = make_pixels(2, 4, 0);
    pixels.value = vec![201.0, 201.0, 200.0, 200.0, 201.0, 201.0, 201.0, 201.0];
    pixels.saturation = vec![29.0, 29.0, 29.0, 29.0, 30.0, 30.0, 29.0, 29.0];

    assert_close(
        white_row_fraction(&pixels, CellBounds { x1: 0, x2: 2 }),
        0.5,
    );
}

#[test]
fn white_fraction_indexes_each_physical_row_by_image_width() {
    let mut pixels = make_pixels(3, 5, 1);
    pixels.value.fill(0.0);
    pixels.saturation.fill(100.0);
    pixels.value[9..12].fill(201.0);
    pixels.saturation[9..12].fill(0.0);

    assert_close(
        white_row_fraction(&pixels, CellBounds { x1: 0, x2: 3 }),
        1.0 / 3.0,
    );
}
