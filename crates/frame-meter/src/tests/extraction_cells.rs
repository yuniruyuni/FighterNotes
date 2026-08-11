//! メーター 1 行分をセルへ切り分けるところに対するテスト。
//!
//! フレームメーターは横一列の 80 個のセルでできている。行の幅を 80 で
//! 割って切り出すので、動画が小さいとセルが 1 ピクセル未満になり、
//! 切り出せないセルが出る。
//!
//! 切り出せなかったセルは「空」として扱う。読めなかったことを縞や
//! 救済読みとして混ぜると、そこに無い技を見ることになる。

use crate::color::{bgr_to_hsv, QuantizedModeScratch};
use crate::constants::CELL_COUNT;
use crate::extraction::cells::{extract, extract_parts};
use crate::extraction::metrics::cell_bounds;
use crate::extraction::source::RowPixels;
use crate::{BrightClass, CellState};

/// 一色で塗られた 1 行分の画素。色は BGR 順。
fn uniform_row(width: usize, height: usize, colour: [u8; 3]) -> RowPixels {
    let trim_y = (height / 6).max(1);
    let patch_height = height - 2 * trim_y;
    let hsv = bgr_to_hsv([colour[0] as f32, colour[1] as f32, colour[2] as f32]);
    RowPixels {
        width,
        height,
        trim_y,
        patch_height,
        region1_rows: (0..patch_height / 2).collect(),
        region2_rows: (patch_height / 2..patch_height).collect(),
        bgr: vec![colour; width * height],
        value: vec![hsv[2]; width * height],
        saturation: vec![hsv[1]; width * height],
    }
}

/// 発生を表す緑（BGR 順）。
const COUNTER: [u8; 3] = [146, 201, 19];

/// 空セルの黒。
const BLACK: [u8; 3] = [23, 20, 23];

/// 先頭から `lit` 個のセルだけを塗り、残りを黒にした 1 行分の画素。
fn row_lit_up_to(width: usize, height: usize, lit: usize, colour: [u8; 3]) -> RowPixels {
    let mut pixels = uniform_row(width, height, BLACK);
    for index in 0..lit {
        let Some(bounds) = cell_bounds(width, index) else {
            continue;
        };
        for row in 0..height {
            for column in bounds.x1..bounds.x2 {
                pixels.bgr[row * width + column] = colour;
            }
        }
    }
    let hsv = bgr_to_hsv([colour[0] as f32, colour[1] as f32, colour[2] as f32]);
    for index in 0..lit {
        let Some(bounds) = cell_bounds(width, index) else {
            continue;
        };
        for row in 0..height {
            for column in bounds.x1..bounds.x2 {
                pixels.value[row * width + column] = hsv[2];
                pixels.saturation[row * width + column] = hsv[1];
            }
        }
    }
    pixels
}

/// 読めた分は全て同じ状態になる。
#[test]
fn a_row_painted_one_colour_reads_as_that_state_everywhere() {
    let observation = extract(uniform_row(1_600, 40, COUNTER));

    assert_eq!(observation.states.len(), CELL_COUNT);
    assert!(observation
        .states
        .iter()
        .all(|state| *state == CellState::Counter));
    assert!(observation.bright.iter().all(|b| *b == BrightClass::Fresh));
}

/// 幅が足りず切り出せないセルは空にする。縞でも救済読みでもない。
#[test]
fn cells_too_narrow_to_cut_are_left_empty() {
    let width = 40;
    let observation = extract(uniform_row(width, 40, COUNTER));

    let unreadable: Vec<usize> = (0..CELL_COUNT)
        .filter(|&index| cell_bounds(width, index).is_none())
        .collect();
    assert!(!unreadable.is_empty(), "この幅では全セルが切り出せている");

    for index in unreadable {
        assert_eq!(observation.states[index], CellState::Empty, "セル {index}");
        assert_eq!(
            observation.bright[index],
            BrightClass::None_,
            "セル {index}"
        );
        assert!(!observation.stripe[index], "セル {index} を縞にしている");
        assert!(
            !observation.rescued[index],
            "セル {index} を救済扱いにしている"
        );
        assert_eq!(observation.v[index], 0.0, "セル {index}");
        assert_eq!(observation.wf[index], 0.0, "セル {index}");
        assert_eq!(observation.quality[index], 0.0, "セル {index}");
        assert_eq!(observation.bgr[index], [0.0; 3], "セル {index}");
    }
}

/// 白の割合はセルごとに測る。縞かどうかの判断材料になる。
#[test]
fn the_white_share_is_measured_for_every_readable_cell() {
    let white = extract(uniform_row(1_600, 40, [236, 233, 233]));
    let green = extract(uniform_row(1_600, 40, COUNTER));

    assert!(
        white.wf.iter().all(|share| *share > 0.9),
        "白を測れていない"
    );
    assert!(
        green.wf.iter().all(|share| *share < 0.1),
        "緑を白と見ている"
    );
}

/// 色が付いたセルの先端を覚える。ここが「今のフレーム」の目印になる。
#[test]
fn the_edge_of_the_freshest_colour_is_recorded() {
    let observation = extract(row_lit_up_to(1_600, 40, 30, COUNTER));

    assert_eq!(
        observation.fresh_edge,
        29,
        "色の先端を指していない: {:?}",
        &observation.states[..32]
    );
}

// ── 数字の切り出し ───────────────────────────────────────────────────────

fn parts(width: usize, height: usize) -> crate::extraction::cells::CellExtraction {
    extract_parts(
        uniform_row(width, height, COUNTER),
        &mut QuantizedModeScratch::new(),
    )
}

/// 数字を読むには、テンプレートが収まるだけの大きさが要る。
#[test]
fn digits_are_only_read_when_the_cell_is_large_enough() {
    assert!(
        parts(1_600, 40).finish_full().digit_corr.is_some(),
        "十分な大きさでも数字を読んでいない"
    );
    assert!(
        parts(1_600, 20).finish_full().digit_corr.is_none(),
        "高さが足りないのに数字を読んでいる"
    );
    assert!(
        parts(800, 40).finish_full().digit_corr.is_none(),
        "幅が足りないのに数字を読んでいる"
    );
}

/// 数字を読まないと決めたときは読まない。
#[test]
fn skipping_the_digits_leaves_them_unread() {
    assert!(parts(1_600, 40)
        .finish_without_digits()
        .digit_corr
        .is_none());
}

/// 指定したセルだけ数字を読む。全部読むと重い。
#[test]
fn only_the_requested_cells_have_their_digits_read() {
    let observation = parts(1_600, 40).finish_sparse([0b1010, 0]);

    assert!(observation.digit_correlation(1).is_some());
    assert!(observation.digit_correlation(3).is_some());
    assert!(observation.digit_correlation(0).is_none());
    assert!(observation.digit_correlation(2).is_none());
    assert!(observation.digit_correlation(64).is_none());
}

#[test]
fn sparse_digits_ignore_bits_beyond_the_eighty_cells() {
    let observation = parts(1_600, 40).finish_sparse([1, 1 << 16]);

    assert!(observation.digit_correlation(0).is_some());
    assert!(observation.digit_correlation(1).is_none());
}

fn patterned_row() -> RowPixels {
    let mut pixels = uniform_row(1_600, 40, COUNTER);
    for row in 0..pixels.height {
        for column in 0..pixels.width {
            pixels.value[row * pixels.width + column] = ((row * 37 + column * 11) % 251) as f32;
        }
    }
    pixels
}

#[test]
fn sparse_digit_patches_keep_the_selected_cell_pixels() {
    let full = extract(patterned_row());
    let sparse =
        extract_parts(patterned_row(), &mut QuantizedModeScratch::new()).finish_sparse([1 << 3, 0]);

    assert_eq!(sparse.digit_correlation(3), full.digit_correlation(3));
    assert!(sparse.digit_correlation(2).is_none());
}

/// 上位の語のビットも読む。64 番目以降のセルを取りこぼさない。
#[test]
fn the_upper_word_of_the_selection_is_honoured() {
    let observation = parts(1_600, 40).finish_sparse([0, 1 << 5]);

    assert!(observation.digit_correlation(69).is_some());
    assert!(observation.digit_correlation(68).is_none());
}

/// 一つも指定しなければ何も読まない。
#[test]
fn an_empty_selection_reads_no_digits() {
    assert!(parts(1_600, 40).finish_sparse([0, 0]).digit_corr.is_none());
}

/// セルの明るさは、そのセルの画素から測る。明るさは状態の判断にも
/// 「今出ている技か、数フレーム前か」の判断にも使う。
#[test]
fn the_brightness_of_each_cell_is_measured_from_its_own_pixels() {
    let lit = extract(row_lit_up_to(1_600, 40, 30, COUNTER));
    let expected = bgr_to_hsv([COUNTER[0] as f32, COUNTER[1] as f32, COUNTER[2] as f32])[2];

    for index in 0..30 {
        assert!(
            (lit.v[index] - expected).abs() < 1.0,
            "セル {index} の明るさを測れていない: {}",
            lit.v[index]
        );
    }
    for index in 30..CELL_COUNT {
        assert!(lit.v[index] < expected, "黒いセルを明るいと測っている");
    }
}

/// 無敵の縞は縞として印を付ける。上下で色が違うことが縞の印。
#[test]
fn a_cell_striped_top_and_bottom_is_marked_as_striped() {
    let width = 1_600;
    let height = 40;
    let mut pixels = uniform_row(width, height, BLACK);
    // 上半分を白、下半分をピンクに塗る = 打撃無敵の縞。
    for row in 0..height {
        let colour = if row < height / 2 {
            [236, 233, 233]
        } else {
            [140, 80, 200]
        };
        let hsv = bgr_to_hsv([colour[0] as f32, colour[1] as f32, colour[2] as f32]);
        for column in 0..width {
            pixels.bgr[row * width + column] = colour;
            pixels.value[row * width + column] = hsv[2];
            pixels.saturation[row * width + column] = hsv[1];
        }
    }

    let observation = extract(pixels);

    assert_eq!(observation.states[10], CellState::InvStrike);
    assert!(observation.stripe[10], "縞に印を付けていない");
    assert!(
        !extract(row_lit_up_to(1_600, 40, 30, COUNTER)).stripe[10],
        "縞でないセルに印を付けている"
    );
}

#[test]
fn rescued_cells_keep_the_rescue_marker_in_the_row_observation() {
    let width = 1_600;
    let height = 40;
    let noisy = [255, 0, 255];
    let mut pixels = uniform_row(width, height, noisy);
    let counter_hsv = bgr_to_hsv([COUNTER[0] as f32, COUNTER[1] as f32, COUNTER[2] as f32]);
    for local_row in (0..6).chain(14..20) {
        let row = pixels.trim_y + local_row;
        for column in 0..width {
            let index = row * width + column;
            pixels.bgr[index] = COUNTER;
            pixels.value[index] = counter_hsv[2];
            pixels.saturation[index] = counter_hsv[1];
        }
    }

    let observation = extract(pixels);
    assert_eq!(observation.states[10], CellState::Counter);
    assert!(observation.rescued[10]);
}

/// フレーム差を示す明るい塊の位置を覚える。色見本に無い明るい色が
/// 塊の印になる。
#[test]
fn the_bright_slab_position_is_remembered() {
    let width = 1_600;
    let height = 40;
    let mut pixels = uniform_row(width, height, BLACK);
    // 色見本に無い明るい色（水色寄りの白）を 1 セルだけ置く。
    let slab = [200u8, 255, 255];
    let hsv = bgr_to_hsv([slab[0] as f32, slab[1] as f32, slab[2] as f32]);
    let bounds = cell_bounds(width, 42).expect("セル 42");
    for row in 0..height {
        for column in bounds.x1..bounds.x2 {
            pixels.bgr[row * width + column] = slab;
            pixels.value[row * width + column] = hsv[2];
            pixels.saturation[row * width + column] = hsv[1];
        }
    }

    let observation = extract(pixels);

    assert_eq!(observation.states[42], CellState::Other);
    assert_eq!(observation.slab_pos, 42, "明るい塊の位置を見失っている");
    assert_eq!(
        extract(uniform_row(1_600, 40, BLACK)).slab_pos,
        -1,
        "塊が無いのに位置を返している"
    );
}

/// 列ごとの平均は、セルが切り出せる幅があるときだけ持つ。
#[test]
fn the_column_means_are_kept_only_when_the_cells_have_width() {
    let wide = extract(uniform_row(1_600, 40, COUNTER));
    let narrow = extract(uniform_row(40, 40, COUNTER));

    assert!(wide.cols.is_some());
    assert!(wide.cols_w > 0);
    assert!(narrow.cols.is_none(), "切り出せない幅で列平均を持っている");
    assert_eq!(narrow.cols_w, 0);
}
