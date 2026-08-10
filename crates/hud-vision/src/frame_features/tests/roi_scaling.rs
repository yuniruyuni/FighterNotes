//! ROI の座標を実解像度へ合わせる変換に対するテスト。
//!
//! ROI の位置は 1920x1080 を基準に書いてある。720p や 1440p の動画では
//! 比例して置き直す。ここがずれると、どの読み取りも別の場所を見る。

use crate::frame_features::scale_roi;

/// 基準の解像度では座標がそのまま通る。
#[test]
fn the_reference_resolution_passes_the_coordinates_through() {
    assert_eq!(scale_roi(172, 853, 64, 95, 1920, 1080), (172, 853, 64, 95));
}

/// 半分の解像度では座標も半分になる。掛けるべきところで割ると、
/// 低解像度で ROI が画面外へ飛ぶ。
#[test]
fn a_smaller_frame_scales_the_coordinates_down() {
    assert_eq!(scale_roi(172, 853, 64, 95, 960, 540), (86, 426, 32, 47));
}

/// 大きい解像度では広がる。
#[test]
fn a_larger_frame_scales_the_coordinates_up() {
    assert_eq!(
        scale_roi(172, 853, 64, 95, 3840, 2160),
        (344, 1706, 128, 190)
    );
}

/// 縦横の比率が違っても、それぞれの軸で独立に合わせる。横長に引き伸ばした
/// 録画で、縦の位置まで動いてはいけない。
#[test]
fn the_two_axes_scale_independently() {
    let (x1, x2, y1, y2) = scale_roi(172, 853, 64, 95, 960, 1080);

    assert_eq!((x1, x2), (86, 426), "横が合っていない");
    assert_eq!((y1, y2), (64, 95), "縦まで動いている");
}

/// 画面の外へは出ない。潰れた寸法でも、後の走査が範囲外を読まないように
/// 収める。
#[test]
fn the_result_stays_inside_the_frame() {
    assert_eq!(scale_roi(172, 853, 64, 95, 1, 1), (0, 0, 0, 0));
    assert_eq!(scale_roi(172, 853, 64, 95, 0, 0), (0, 0, 0, 0));
    assert_eq!(scale_roi(0, 4000, 0, 4000, 1920, 1080), (0, 1920, 0, 1080));
}
