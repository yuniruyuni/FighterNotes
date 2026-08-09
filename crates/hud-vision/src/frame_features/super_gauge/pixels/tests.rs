//! パッチの範囲判定と画素の見方に対するテスト。
//!
//! ここが緩むと切り詰められたバッファで範囲外を読む。厳しすぎると
//! 正しく届いたフレームを丸ごと捨てる。

use super::*;

const FRAME_WIDTH: usize = 64;
const FRAME_HEIGHT: usize = 32;

fn frame() -> Vec<u8> {
    vec![0u8; FRAME_WIDTH * FRAME_HEIGHT * 4]
}

fn patch(x: usize, y: usize, width: usize, height: usize) -> Patch {
    Patch {
        x,
        y,
        width,
        height,
    }
}

/// 絵に収まっているパッチは通す。右下の隅にぴったり接する場合も含む。
#[test]
fn a_patch_inside_the_frame_fits() {
    assert!(patch_fits(&frame(), FRAME_WIDTH, patch(0, 0, 8, 8)));
    assert!(patch_fits(
        &frame(),
        FRAME_WIDTH,
        patch(FRAME_WIDTH - 8, FRAME_HEIGHT - 8, 8, 8)
    ));
}

/// 右へ 1px でもはみ出したら通さない。
#[test]
fn a_patch_past_the_right_edge_does_not_fit() {
    assert!(!patch_fits(
        &frame(),
        FRAME_WIDTH,
        patch(FRAME_WIDTH - 8, 0, 9, 8)
    ));
}

/// 下へはみ出す場合も同じ。バッファの長さから高さを割り出す。
#[test]
fn a_patch_past_the_bottom_edge_does_not_fit() {
    assert!(!patch_fits(
        &frame(),
        FRAME_WIDTH,
        patch(0, FRAME_HEIGHT - 8, 8, 9)
    ));
}

/// 切り詰められたバッファは、宣言された高さに届かない。
#[test]
fn a_truncated_buffer_does_not_fit() {
    let half = vec![0u8; FRAME_WIDTH * (FRAME_HEIGHT / 2) * 4];

    assert!(!patch_fits(
        &half,
        FRAME_WIDTH,
        patch(0, FRAME_HEIGHT - 8, 8, 8)
    ));
}

/// 潰れた寸法は通さない。0 幅のパッチを読むと空の集計から値が出る。
#[test]
fn a_degenerate_patch_does_not_fit() {
    assert!(!patch_fits(&frame(), 0, patch(0, 0, 8, 8)));
    assert!(!patch_fits(&frame(), FRAME_WIDTH, patch(0, 0, 0, 8)));
    assert!(!patch_fits(&frame(), FRAME_WIDTH, patch(0, 0, 8, 0)));
}

/// 画素は行優先で並ぶ。ここがずれると絵全体が斜めに読める。
#[test]
fn a_pixel_is_read_from_its_row_and_column() {
    let mut rgba = frame();
    let index = (3 * FRAME_WIDTH + 5) * 4;
    rgba[index] = 10;
    rgba[index + 1] = 20;
    rgba[index + 2] = 30;

    assert_eq!(rgb_at(&rgba, FRAME_WIDTH, 5, 3), [10, 20, 30]);
    assert_eq!(rgb_at(&rgba, FRAME_WIDTH, 3, 5), [0, 0, 0]);
}

/// グリフの塗りは三色とも明るい。淡い色は文字ではない。
#[test]
fn only_a_bright_neutral_pixel_counts_as_glyph_white() {
    assert!(is_glyph_white([190, 190, 190]));
    assert!(!is_glyph_white([189, 255, 255]));
    assert!(!is_glyph_white([255, 189, 255]));
    assert!(!is_glyph_white([255, 255, 189]));
}

/// 隣は上下左右の四つ。格子の外へは出ない。
#[test]
fn the_neighbours_of_an_interior_cell_are_the_four_sides() {
    let mut found: Vec<_> = neighbors(1, 1, 3, 3).collect();
    found.sort_unstable();

    assert_eq!(found, vec![1, 3, 5, 7]);
}

/// 端の格子では、外側の隣を返さない。返すと別の行へ回り込む。
#[test]
fn a_corner_cell_has_only_the_neighbours_inside_the_grid() {
    let mut top_left: Vec<_> = neighbors(0, 0, 3, 3).collect();
    top_left.sort_unstable();
    let mut bottom_right: Vec<_> = neighbors(2, 2, 3, 3).collect();
    bottom_right.sort_unstable();

    assert_eq!(top_left, vec![1, 3]);
    assert_eq!(bottom_right, vec![5, 7]);
}
