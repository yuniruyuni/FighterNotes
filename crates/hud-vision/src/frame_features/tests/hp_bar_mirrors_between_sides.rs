//! 左右の HP バーが鏡像であることに対するテスト。
//!
//! 二つの ROI は画面の中心について対称で、バーの傾きも減る向きも逆。
//! 片側だけを試していると、反対側が別の端から数えていても気づけない。

use super::support::*;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

/// 同じ絵を映せば、両側が同じ残量を読む。
#[test]
fn the_two_sides_read_the_same_mirrored_bar() {
    for fill in [0.25_f32, 0.5, 0.75, 1.0] {
        let first = make_rgba_p1_bar(fill);
        let second = mirror_frame_for_p2(&first);

        let (left, left_uncertain) = hp_fill_ratio_with_quality(&first, WIDTH, HEIGHT, "p1");
        let (right, right_uncertain) = hp_fill_ratio_with_quality(&second, WIDTH, HEIGHT, "p2");

        assert!(!left_uncertain, "{fill}: P1 のバーを読めていない");
        assert!(!right_uncertain, "{fill}: P2 のバーを読めていない");
        assert!(
            (left - right).abs() < 0.01,
            "{fill}: 左 {left} と右 {right} が食い違う"
        );
    }
}

/// 映した絵を反対側の読みに掛けても値は出ない。ROI の位置が違う。
#[test]
fn a_mirrored_frame_does_not_read_on_the_original_side() {
    let mirrored = mirror_frame_for_p2(&make_rgba_p1_bar(0.5));

    let (_, uncertain) = hp_fill_ratio_with_quality(&mirrored, WIDTH, HEIGHT, "p1");

    assert!(uncertain, "P2 に映した絵を P1 が読んでいる");
}

/// 列ごとの判定も鏡像になる。P1 で埋まっている列は、映した絵では
/// 反対の端から数えて同じ位置が埋まっている。
#[test]
fn the_active_columns_mirror_too() {
    let first = make_rgba_p1_bar(0.5);
    let second = mirror_frame_for_p2(&first);

    let left = hp_col_active(&first, WIDTH, HEIGHT, "p1");
    let right = hp_col_active(&second, WIDTH, HEIGHT, "p2");
    let right_reversed: Vec<bool> = right.into_iter().rev().collect();

    let disagreements = left
        .iter()
        .zip(&right_reversed)
        .filter(|(a, b)| a != b)
        .count();

    assert_eq!(left.len(), right_reversed.len(), "列数が食い違う");
    assert!(
        disagreements < 8,
        "鏡像のはずの列が {disagreements} 本食い違う"
    );
}
