use crate::digits::digit_correlations;
use crate::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W};

use super::assert_close;

const STRIDE: usize = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;

fn templates() -> Vec<f32> {
    include_bytes!("../data/meter_digits.bin")
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect()
}

fn shifted(template: &[f32], dy: i32, dx: i32) -> (Vec<f32>, f32) {
    let source_y = dy.max(0) as usize;
    let template_y = (-dy).max(0) as usize;
    let rows = DIGIT_TEMPLATE_H - dy.unsigned_abs() as usize;
    let source_x = dx.max(0) as usize;
    let template_x = (-dx).max(0) as usize;
    let columns = DIGIT_TEMPLATE_W - dx.unsigned_abs() as usize;
    let mut patch = vec![0.0; STRIDE];
    let mut energy = 0.0;

    for row in 0..rows {
        for column in 0..columns {
            let source_index = (source_y + row) * DIGIT_TEMPLATE_W + source_x + column;
            let template_index = (template_y + row) * DIGIT_TEMPLATE_W + template_x + column;
            patch[source_index] = template[template_index];
            energy += template[template_index] * template[template_index];
        }
    }
    (patch, energy / (rows * columns) as f32)
}

#[test]
fn correlations_reject_wrong_patch_count() {
    assert_eq!(digit_correlations(&[]), None);
    assert_eq!(
        digit_correlations(&vec![0.0; CELL_COUNT * STRIDE - 1]),
        None
    );
}

#[test]
fn correlations_align_templates_in_both_directions() {
    let templates = templates();
    let mut cells = vec![0.0; CELL_COUNT * STRIDE];
    let cases = [(0, 0, 0), (4, 1, 2), (8, -1, -2), (6, 0, -1)];
    let mut expected = Vec::new();

    for (cell, &(digit, dy, dx)) in cases.iter().enumerate() {
        let template = &templates[digit * STRIDE..(digit + 1) * STRIDE];
        let (patch, energy) = shifted(template, dy, dx);
        cells[cell * STRIDE..(cell + 1) * STRIDE].copy_from_slice(&patch);
        expected.push((digit, energy));
    }

    let correlations = digit_correlations(&cells).unwrap();
    assert_eq!(correlations.len(), CELL_COUNT);
    for (cell, (digit, energy)) in expected.into_iter().enumerate() {
        let scores = correlations[cell];
        let best = scores
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .unwrap()
            .0;
        if cell == 0 {
            assert_eq!(best, digit, "cell {cell}: {scores:?}");
        }
        assert_close(scores[digit], energy);
    }
    assert_eq!(correlations[10], [0.0; 10]);
}
