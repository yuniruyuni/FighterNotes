use std::sync::OnceLock;

use crate::constants::{CELL_COUNT, DIGIT_TEMPLATE_H, DIGIT_TEMPLATE_W};

// 実ゲームを撮影した動画サンプルから生成した、セルごとの正規化済み画素統計。
// 元動画、frame、cropは含まず、数字相関に必要なf32値だけを保持する。
static DIGIT_TEMPLATE_BYTES: &[u8] = include_bytes!("data/meter_digits.bin");

fn digit_templates() -> Option<&'static [f32]> {
    let expected = 10 * DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W * 4;
    if DIGIT_TEMPLATE_BYTES.len() != expected {
        return None;
    }
    static CACHE: OnceLock<Vec<f32>> = OnceLock::new();
    let templates = CACHE.get_or_init(|| {
        DIGIT_TEMPLATE_BYTES
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect()
    });
    Some(templates.as_slice())
}

pub(crate) fn digit_correlations(cells_vpatch: &[f32]) -> Option<Vec<[f32; 10]>> {
    let stride = DIGIT_TEMPLATE_H * DIGIT_TEMPLATE_W;
    if cells_vpatch.len() != CELL_COUNT * stride {
        return None;
    }
    let templates = digit_templates()?;
    let height = DIGIT_TEMPLATE_H;
    let width = DIGIT_TEMPLATE_W;
    let mut best = vec![[-1.0f32; 10]; CELL_COUNT];

    for dy in [-1i32, 0, 1] {
        let source_y = dy.max(0) as usize;
        let source_y_end = if dy >= 0 {
            height
        } else {
            height - (-dy) as usize
        };
        let template_y = (-dy).max(0) as usize;
        let row_count = source_y_end - source_y;
        if row_count == 0 {
            continue;
        }

        for dx in [-2i32, -1, 0, 1, 2] {
            let source_x = dx.max(0) as usize;
            let source_x_end = if dx >= 0 {
                width
            } else {
                width - (-dx) as usize
            };
            let template_x = (-dx).max(0) as usize;
            let column_count = source_x_end - source_x;
            if column_count == 0 {
                continue;
            }

            let area = (row_count * column_count) as f32;
            for cell_index in 0..CELL_COUNT {
                let patch = &cells_vpatch[cell_index * stride..(cell_index + 1) * stride];
                for (template_index, best_score) in best[cell_index].iter_mut().enumerate() {
                    let mut dot = 0.0f32;
                    for row in 0..row_count {
                        for column in 0..column_count {
                            let source_index = (source_y + row) * width + (source_x + column);
                            let template_index = template_index * height * width
                                + (template_y + row) * width
                                + (template_x + column);
                            dot += patch[source_index] * templates[template_index];
                        }
                    }
                    let correlation = dot / area;
                    *best_score = best_score.max(correlation);
                }
            }
        }
    }
    Some(best)
}
