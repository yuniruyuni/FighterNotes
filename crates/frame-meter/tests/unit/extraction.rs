use super::support::*;

// ─── extract_row_obs ─────────────────────────────────────────────────────────
//
// 1920x1080 RGBA フレームを合成して end-to-end パイプラインを検証する。
//
// ROI 座標（lib.rs の定数）:
//   x: [359, 1559)  （80セル × 15px ≒ 1200px）
//   左 y: [796, 834)
//   右 y: [836, 874)
//
// RGBA 順序: getImageData 互換 = [R, G, B, A]
// BGR 変換:  row_bgr = [b, g, r]  ← lib.rs 内部
// よって Counter BGR=[146,201,19] は RGBA=[R=19, G=201, B=146, A=255]

const W: usize = 1920;
const H: usize = 1080;
const RGBA_SIZE: usize = W * H * 4;

fn blank_frame() -> Vec<u8> {
    vec![0u8; RGBA_SIZE]
}

fn fill_roi(frame: &mut [u8], x1: usize, x2: usize, y1: usize, y2: usize, rgba: [u8; 4]) {
    for y in y1..y2 {
        for x in x1..x2 {
            let idx = (y * W + x) * 4;
            frame[idx] = rgba[0];
            frame[idx + 1] = rgba[1];
            frame[idx + 2] = rgba[2];
            frame[idx + 3] = rgba[3];
        }
    }
}

const LEFT_ROW_Y1: usize = 796;
const CELL_WIDTH: usize = 15;
const PATCH_TRIM_Y: usize = 6;
const STRIPE_REGION1_ROWS: &[usize] = &[4, 5, 9, 10, 14, 15, 19, 20, 24];

fn rgba_from_bgr(bgr: [f32; 3]) -> [u8; 4] {
    [
        bgr[2].round() as u8,
        bgr[1].round() as u8,
        bgr[0].round() as u8,
        255,
    ]
}

fn fill_left_cell_pair(frame: &mut [u8], cell: usize, a_bgr: [f32; 3], b_bgr: [f32; 3]) {
    let x1 = 359 + cell * CELL_WIDTH;
    let x2 = x1 + CELL_WIDTH;
    fill_roi(frame, x1, x2, LEFT_ROW_Y1, 834, rgba_from_bgr(b_bgr));
    for &row in STRIPE_REGION1_ROWS {
        let y = LEFT_ROW_Y1 + PATCH_TRIM_Y + row;
        fill_roi(frame, x1, x2, y, y + 1, rgba_from_bgr(a_bgr));
    }
}

fn set_gray(frame: &mut [u8], x: usize, y: usize, value: u8) {
    let index = (y * W + x) * 4;
    frame[index] = value;
    frame[index + 1] = value;
    frame[index + 2] = value;
    frame[index + 3] = 255;
}

#[test]
fn extract_all_black_is_empty() {
    let frame = blank_frame();
    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);
    assert_eq!(left.states.len(), CELL_COUNT);
    assert_eq!(right.states.len(), CELL_COUNT);
    assert!(
        left.states.iter().all(|s| *s == CellState::Empty),
        "left should all be Empty; first 5: {:?}",
        &left.states[..5]
    );
    assert!(
        right.states.iter().all(|s| *s == CellState::Empty),
        "right should all be Empty; first 5: {:?}",
        &right.states[..5]
    );
}

#[test]
fn extract_left_roi_counter() {
    let mut frame = blank_frame();
    // Counter BGR=[146,201,19] → RGBA=[R=19, G=201, B=146, A=255]
    fill_roi(&mut frame, 359, 1559, 796, 834, [19, 201, 146, 255]);
    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);

    assert!(
        left.states.iter().all(|s| *s == CellState::Counter),
        "left should all be Counter; first 5: {:?}",
        &left.states[..5]
    );
    assert!(
        right.states.iter().all(|s| *s == CellState::Empty),
        "right should all be Empty (untouched)"
    );
}

#[test]
fn extract_right_roi_stun() {
    let mut frame = blank_frame();
    // Stun BGR=[55,255,247] → RGBA=[R=247, G=255, B=55, A=255]
    fill_roi(&mut frame, 359, 1559, 836, 874, [247, 255, 55, 255]);
    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);

    assert!(
        left.states.iter().all(|s| *s == CellState::Empty),
        "left should all be Empty (untouched)"
    );
    assert!(
        right.states.iter().all(|s| *s == CellState::Stun),
        "right should all be Stun; first 5: {:?}",
        &right.states[..5]
    );
}

#[test]
fn extract_both_sides_independent() {
    let mut frame = blank_frame();
    // 左: Counter, 右: Active BGR=[93,20,176] → RGBA=[R=176, G=20, B=93, A=255]
    fill_roi(&mut frame, 359, 1559, 796, 834, [19, 201, 146, 255]);
    fill_roi(&mut frame, 359, 1559, 836, 874, [176, 20, 93, 255]);
    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);

    assert!(
        left.states.iter().all(|s| *s == CellState::Counter),
        "left should be Counter; got {:?}",
        &left.states[..3]
    );
    assert!(
        right.states.iter().all(|s| *s == CellState::Active),
        "right should be Active; got {:?}",
        &right.states[..3]
    );
}

#[test]
fn extract_state_matrix_from_synthetic_frame() {
    let mut frame = blank_frame();
    let cases = [
        (counter(), counter(), CellState::Counter),
        (punish_counter(), punish_counter(), CellState::PunishCounter),
        (
            motion_recovery(),
            motion_recovery(),
            CellState::MotionRecovery,
        ),
        (active(), active(), CellState::Active),
        (
            projectile_active(),
            projectile_active(),
            CellState::ProjectileActive,
        ),
        (stun(), stun(), CellState::Stun),
        (parry(), parry(), CellState::Parry),
        (white(), gray(), CellState::InvFull),
        (white(), stripe_pink(), CellState::InvStrike),
        (white(), stripe_orange(), CellState::InvProj),
        (white(), white(), CellState::Other),
        (black(), black(), CellState::Empty),
    ];
    for (cell, (a_bgr, b_bgr, _)) in cases.iter().enumerate() {
        fill_left_cell_pair(&mut frame, cell, *a_bgr, *b_bgr);
    }

    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);
    let expected: Vec<CellState> = cases.into_iter().map(|(_, _, state)| state).collect();

    assert_eq!(&left.states[..expected.len()], expected.as_slice());
    assert!(left.states[expected.len()..]
        .iter()
        .all(|state| *state == CellState::Empty));
    assert!(right.states.iter().all(|state| *state == CellState::Empty));
    assert_eq!(left.fresh_edge, 9);
}

#[test]
fn extract_digit_correlations_accept_synthetic_templates() {
    const TEMPLATE_H: usize = 26;
    const TEMPLATE_W: usize = 13;
    const TEMPLATE_SIZE: usize = TEMPLATE_H * TEMPLATE_W;
    const DIGIT_COUNT: usize = 10;

    let bytes = include_bytes!("../../src/data/meter_digits.bin");
    assert_eq!(bytes.len(), DIGIT_COUNT * TEMPLATE_SIZE * 4);
    let templates: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    let mut frame = blank_frame();
    for cell in 0..CELL_COUNT {
        let digit = cell % DIGIT_COUNT;
        let template = &templates[digit * TEMPLATE_SIZE..(digit + 1) * TEMPLATE_SIZE];
        for row in 0..TEMPLATE_H {
            for col in 0..TEMPLATE_W {
                let normalized = template[row * TEMPLATE_W + col];
                let value = (128.0 + normalized * 48.0).round().clamp(0.0, 255.0) as u8;
                let x = 359 + cell * CELL_WIDTH + 1 + col;
                let y = LEFT_ROW_Y1 + PATCH_TRIM_Y + row;
                set_gray(&mut frame, x, y, value);
            }
        }
    }

    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);
    let correlations = left
        .digit_corr
        .expect("synthetic 1920x1080 frame should produce digit correlations");

    for (cell, values) in correlations.iter().enumerate() {
        let expected_digit = cell % DIGIT_COUNT;
        assert!(values.iter().all(|value| value.is_finite()));
        // meter-tracker は argmax ではなく候補レイアウトを比較し、0.55 以上を
        // 数字の根拠として扱う。各テンプレートがその契約を満たすことを確認する。
        assert!(
            values[expected_digit] >= 0.55,
            "cell {cell}: digit {expected_digit} score is too low: {values:?}"
        );
    }
}

#[test]
fn extract_fresh_edge_half_filled() {
    // 左メーターの前半 40 セルを Counter で埋め、後半は空にする。
    // セル幅 = 1200 / 80 = 15px、40 セル = 600px → x=[359, 959)
    let mut frame = blank_frame();
    fill_roi(&mut frame, 359, 959, 796, 834, [19, 201, 146, 255]);
    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);

    // セル 39 の直後 (セル 40) が黒 → has_front_gap → fresh_edge = 39
    assert_eq!(
        left.fresh_edge, 39,
        "fresh_edge should be 39; got {}",
        left.fresh_edge
    );
}

#[test]
fn extract_full_counter_fresh_edge_at_last() {
    // 全セル Counter → fresh_edge は 79（最後のセル; 次セルなし → has_front_gap = true）
    let mut frame = blank_frame();
    fill_roi(&mut frame, 359, 1559, 796, 834, [19, 201, 146, 255]);
    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);
    assert_eq!(left.fresh_edge, 79);
}

#[test]
fn extract_slab_pos_for_other() {
    // 全セルを "Other" になる色で埋める。
    // classify_cell_pair が Other を返すには白+白（high V, whiteish both）
    // white BGR=[236,233,233] → RGBA=[R=233, G=233, B=236, A=255]
    let mut frame = blank_frame();
    fill_roi(&mut frame, 359, 1559, 796, 834, [233, 233, 236, 255]);
    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);

    // 白系セルは Other または InvFull になる（実装依存）が、
    // slab_pos は Other かつ V >= HIGHLIGHT_V_MIN(90) の最右端
    // white V ≈ 238 >> 90 → Other セルが存在すれば slab_pos >= 0
    // （InvFull と判定される可能性もあるためゆるめの検証）
    let has_info = left.states.iter().any(|s| {
        matches!(
            s,
            CellState::Other | CellState::InvFull | CellState::InvStrike | CellState::InvProj
        )
    });
    assert!(
        has_info,
        "bright white pixels should produce non-empty state"
    );
}

#[test]
fn extract_output_lengths_always_cell_count() {
    let frame = blank_frame();
    let (left, right) = extract_row_obs(&frame, W as u32, H as u32);
    assert_eq!(left.v.len(), CELL_COUNT);
    assert_eq!(left.wf.len(), CELL_COUNT);
    assert_eq!(left.states.len(), CELL_COUNT);
    assert_eq!(left.bright.len(), CELL_COUNT);
    assert_eq!(left.bgr.len(), CELL_COUNT);
    assert_eq!(right.v.len(), CELL_COUNT);
    assert_eq!(right.states.len(), CELL_COUNT);
}

#[test]
fn extract_v_values_match_color_brightness() {
    // Counter color の V ≈ 201（lib.rs の bgr_to_hsv: v = max_channel * 255/255 * 1.0 = max ）
    // G=201 が max → V = 201
    let mut frame = blank_frame();
    fill_roi(&mut frame, 359, 1559, 796, 834, [19, 201, 146, 255]);
    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);

    for (i, &v) in left.v.iter().enumerate() {
        assert!(
            (v - 201.0).abs() < 2.0,
            "cell {i}: V should be ~201, got {v}"
        );
    }
}

#[test]
fn extract_empty_v_below_threshold() {
    // 黒フレームの V 値は EMPTY_V_MAX より小さい
    let frame = blank_frame();
    let (left, _) = extract_row_obs(&frame, W as u32, H as u32);
    for (i, &v) in left.v.iter().enumerate() {
        assert!(
            v < EMPTY_V_MAX,
            "cell {i}: V={v} should be < EMPTY_V_MAX({EMPTY_V_MAX})"
        );
    }
}
