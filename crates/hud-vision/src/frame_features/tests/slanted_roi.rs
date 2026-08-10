//! 平行四辺形の ROI と画面座標の対応に対するテスト。
//!
//! HP バーもドライブゲージも平行四辺形で、行が下がるごとに横へずれる。
//! この対応がずれると、どの読み取りも隣の列を見る。読みは「それらしい値」
//! のまま返るので、結果からは気づけない。

use crate::frame_features::SlantedRoi;

const FRAME_WIDTH: usize = 64;
const FRAME_HEIGHT: usize = 32;

fn frame() -> Vec<u8> {
    let mut rgba = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT * 4];
    // 各画素に、その位置が判る色を置く。
    for y in 0..FRAME_HEIGHT {
        for x in 0..FRAME_WIDTH {
            let index = (y * FRAME_WIDTH + x) * 4;
            rgba[index] = x as u8;
            rgba[index + 1] = y as u8;
            rgba[index + 2] = 255;
            rgba[index + 3] = 255;
        }
    }
    rgba
}

fn roi(rgba: &[u8], slope: f32) -> SlantedRoi<'_> {
    SlantedRoi {
        rgba,
        frame_width: FRAME_WIDTH,
        x: 10..30,
        y_start: 4,
        height: 16,
        strip_y: 0,
        slope,
    }
}

/// 傾きが無ければ、列はまっすぐ下りる。
#[test]
fn a_flat_roi_reads_straight_down() {
    let rgba = frame();
    let roi = roi(&rgba, 0.0);

    for row in 0..16usize {
        assert_eq!(roi.column_x(5, row, 0), Some(15), "{row} 行目がずれている");
    }
}

/// 行が下がるごとに、傾きの分だけ横へずれる。端数は四捨五入する。
#[test]
fn each_row_shifts_by_the_slope() {
    let rgba = frame();
    let roi = roi(&rgba, 0.75);

    assert_eq!(roi.column_x(5, 0, 0), Some(15));
    assert_eq!(roi.column_x(5, 1, 0), Some(16));
    assert_eq!(roi.column_x(5, 2, 0), Some(17), "1.5 を切り捨てている");
    assert_eq!(roi.column_x(5, 4, 0), Some(18));
}

/// 傾きは逆向きにもなる。左右のバーで向きが反転する。
#[test]
fn a_negative_slope_shifts_the_other_way() {
    let rgba = frame();
    let roi = roi(&rgba, -0.75);

    assert_eq!(roi.column_x(5, 0, 0), Some(15));
    assert_eq!(roi.column_x(5, 4, 0), Some(12));
}

/// ずれの起点は走査を始める行。上のふちどりを飛ばした分まで
/// ずらすと、列全体が横へずれる。
#[test]
fn the_shift_is_measured_from_the_first_scanned_row() {
    let rgba = frame();
    let roi = roi(&rgba, 0.75);

    assert_eq!(roi.column_x(5, 4, 4), Some(15), "起点の行がずれている");
    assert_eq!(roi.column_x(5, 8, 4), Some(18));
}

/// 起点より手前の行は、その列に属さない。
#[test]
fn rows_before_the_origin_belong_to_no_column() {
    let rgba = frame();

    assert_eq!(roi(&rgba, 0.75).column_x(5, 3, 4), None);
}

/// ずれた先が ROI の外へ出た行は、その列に属さない。読むと隣の
/// バーや背景を数えることになる。
#[test]
fn rows_pushed_out_of_the_roi_belong_to_no_column() {
    let rgba = frame();

    // 右端の列を右へずらす。
    assert_eq!(roi(&rgba, 0.75).column_x(19, 0, 0), Some(29));
    assert_eq!(
        roi(&rgba, 0.75).column_x(19, 2, 0),
        None,
        "ROI の右へ出た行"
    );
    // 左端の列を左へずらす。
    assert_eq!(roi(&rgba, -0.75).column_x(0, 0, 0), Some(10));
    assert_eq!(
        roi(&rgba, -0.75).column_x(0, 2, 0),
        None,
        "ROI の左へ出た行"
    );
}

/// 読む画素は、その列とその行の交点。行と列を取り違えると絵が転置する。
#[test]
fn the_pixel_comes_from_the_intersection() {
    let rgba = frame();
    let roi = roi(&rgba, 0.75);

    // 列 5・行 2 → x=17、画面上の y は y_start + row = 6。
    assert_eq!(roi.rgb_at(5, 2, 0), Some([17.0, 6.0, 255.0]));
}

/// 帯だけを渡されたときは、帯の先頭行の分だけ上へ詰める。
#[test]
fn a_strip_offsets_the_rows_it_was_cut_from() {
    let rgba = frame();
    let mut strip = roi(&rgba, 0.0);
    strip.strip_y = 4;

    // y_start=4、strip_y=4 なので、行 0 は帯の 0 行目。
    assert_eq!(strip.rgb_at(5, 0, 0), Some([15.0, 0.0, 255.0]));
}

/// 帯の先頭より上の行は読めない。切り出す前の座標をそのまま渡された
/// 場合に、負の位置を読まないため。
#[test]
fn rows_above_the_strip_are_unreadable() {
    let rgba = frame();
    let mut strip = roi(&rgba, 0.0);
    strip.strip_y = 10;

    assert_eq!(strip.rgb_at(5, 0, 0), None);
}

/// バッファの終わりを越える行も読めない。切り詰められた入力で範囲外を
/// 触らないため。
#[test]
fn rows_past_the_end_of_the_buffer_are_unreadable() {
    let short = vec![0u8; FRAME_WIDTH * 6 * 4];
    let roi = roi(&short, 0.0);

    assert!(roi.rgb_at(5, 0, 0).is_some(), "届いている行を捨てている");
    assert_eq!(roi.rgb_at(5, 4, 0), None, "バッファの外を読んでいる");
}
