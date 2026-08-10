//! HP バーの ROI に「それらしい絵」があるかの度合いに対するテスト。
//!
//! 試合画面かどうかの判定に使う。ROI が鮮やかで明るい画素で埋まっていれば
//! HP バーが映っている見込みが高く、暗くくすんでいればメニューや暗転である。
//!
//! ここが常に高い値を返すと、試合でない場面まで解析対象になる。逆に常に
//! 低いと、試合が丸ごと落ちる。

use super::support::hud_strip_from_frame;
use crate::frame_features::{hp_bar_score, hp_bar_score_from_hud_strip};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

/// ROI 全体を指定色で塗った 1 フレーム。走査は矩形なので傾きは要らない。
fn frame_with_roi(rgb: (u8, u8, u8)) -> Vec<u8> {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 64..95usize {
        for gx in 172..853usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index] = rgb.0;
            rgba[index + 1] = rgb.1;
            rgba[index + 2] = rgb.2;
            rgba[index + 3] = 255;
        }
    }
    rgba
}

/// 鮮やかで明るい画素で埋まっていれば、ほぼ全面が該当する。
#[test]
fn a_vivid_bar_fills_the_score() {
    let score = hp_bar_score(&frame_with_roi((220, 20, 20)), WIDTH, HEIGHT, "p1");

    assert!(score > 0.95, "鮮やかなバーを拾えていない: {score}");
}

/// 暗転や黒帯では 0 に近い。ここが高いと、試合でない場面を解析してしまう。
#[test]
fn a_dark_screen_scores_near_zero() {
    let score = hp_bar_score(&frame_with_roi((5, 5, 5)), WIDTH, HEIGHT, "p1");

    assert!(score < 0.05, "暗転をバーと読んでいる: {score}");
}

/// 明るくても彩度の無い灰色は該当しない。白い背景のメニューを
/// 試合画面と取り違えないため。
#[test]
fn a_bright_but_colourless_screen_is_not_a_bar() {
    let score = hp_bar_score(&frame_with_roi((200, 200, 200)), WIDTH, HEIGHT, "p1");

    assert!(score < 0.05, "無彩色をバーと読んでいる: {score}");
}

/// 彩度が高くても暗ければ該当しない。暗い背景の色味を拾わないため。
#[test]
fn a_saturated_but_dark_colour_is_not_a_bar() {
    let score = hp_bar_score(&frame_with_roi((60, 5, 5)), WIDTH, HEIGHT, "p1");

    assert!(score < 0.05, "暗い色味をバーと読んでいる: {score}");
}

/// 半分だけ映っていれば、およそ半分の値になる。割合として意味を持って
/// いなければ、判定の閾値が効かない。
#[test]
fn a_half_covered_roi_scores_about_half() {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 64..95usize {
        for gx in 172..512usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index] = 220;
            rgba[index + 1] = 20;
            rgba[index + 2] = 20;
        }
    }

    let score = hp_bar_score(&rgba, WIDTH, HEIGHT, "p1");

    assert!(
        (0.40..=0.60).contains(&score),
        "割合になっていない: {score}"
    );
}

/// 左右の ROI は別の場所。片側に塗った絵で反対側が高くなってはいけない。
#[test]
fn the_two_sides_look_at_different_places() {
    let painted_for_p1 = frame_with_roi((220, 20, 20));

    assert!(hp_bar_score(&painted_for_p1, WIDTH, HEIGHT, "p1") > 0.95);
    assert!(
        hp_bar_score(&painted_for_p1, WIDTH, HEIGHT, "p2") < 0.05,
        "P1 に塗った絵で P2 が反応している"
    );
}

/// 潰れた画面や短いバッファでは 0 を返す。0 除算と範囲外参照の手前で止める。
#[test]
fn a_degenerate_input_scores_zero() {
    assert_eq!(hp_bar_score(&[], 0, 0, "p1"), 0.0);
    assert_eq!(hp_bar_score(&[255u8; 100], WIDTH, HEIGHT, "p1"), 0.0);
}

/// 帯だけを渡しても、全画面と同じ値になる。browser は帯だけを渡すので、
/// ここがずれると試合の検出が食い違う。
#[test]
fn the_hud_strip_scores_the_same_as_the_whole_frame() {
    let full = frame_with_roi((220, 20, 20));
    let strip = hud_strip_from_frame(&full);

    let from_full = hp_bar_score(&full, WIDTH, HEIGHT, "p1");
    let from_strip = hp_bar_score_from_hud_strip(&strip, WIDTH, HEIGHT, "p1");

    assert!(
        (from_full - from_strip).abs() < 1e-6,
        "全画面 {from_full} と帯 {from_strip} が食い違う"
    );
}

/// ROI の外は見ない。一行でも余分に見れば、満たした ROI でも 1 に
/// 届かなくなる。
#[test]
fn a_fully_painted_roi_scores_exactly_one() {
    let score = hp_bar_score(&frame_with_roi((220, 20, 20)), WIDTH, HEIGHT, "p1");

    assert_eq!(score, 1.0, "ROI の外まで数えている");
}

/// ROI の一行が欠けていれば、その分だけ下がる。ROI が狭すぎれば
/// 欠けに気づけない。
#[test]
fn a_missing_row_lowers_the_score() {
    let mut rgba = frame_with_roi((220, 20, 20));
    for gx in 172..853usize {
        let index = (94 * WIDTH as usize + gx) * 4;
        rgba[index..index + 3].fill(0);
    }

    let score = hp_bar_score(&rgba, WIDTH, HEIGHT, "p1");

    assert!(score < 1.0, "ROI の最終行を見ていない");
    assert!(score > 0.9, "一行の欠けで大きく落ちすぎている");
}

/// 彩度の閾値はちょうどでは足りない。仕様そのものなので、両側から留める。
#[test]
fn the_saturation_threshold_needs_to_be_exceeded() {
    let at_the_edge = hp_bar_score(&frame_with_roi((255, 210, 210)), WIDTH, HEIGHT, "p1");
    let just_over = hp_bar_score(&frame_with_roi((255, 209, 209)), WIDTH, HEIGHT, "p1");

    assert_eq!(at_the_edge, 0.0, "ちょうどの彩度を数えている");
    assert_eq!(just_over, 1.0, "超えた彩度を数えていない");
}

/// 明度の閾値も同じ。
#[test]
fn the_brightness_threshold_needs_to_be_exceeded() {
    let at_the_edge = hp_bar_score(&frame_with_roi((80, 0, 0)), WIDTH, HEIGHT, "p1");
    let just_over = hp_bar_score(&frame_with_roi((81, 0, 0)), WIDTH, HEIGHT, "p1");

    assert_eq!(at_the_edge, 0.0, "ちょうどの明度を数えている");
    assert_eq!(just_over, 1.0, "超えた明度を数えていない");
}

/// 画素は 4 byte ずつ並ぶ。読む位置がずれると、隣の画素の成分を
/// 混ぜて色を作ることになる。
#[test]
fn each_pixel_is_read_from_its_own_four_bytes() {
    // 1 列おきに「鮮やかな赤」と「暗い緑」を並べる。位置がずれると
    // 二つが混ざり、割合が半分から動く。
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 64..95usize {
        for gx in 172..853usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            let colour = if gx % 2 == 0 {
                [220u8, 20, 20]
            } else {
                [0, 40, 0]
            };
            rgba[index..index + 3].copy_from_slice(&colour);
            rgba[index + 3] = 255;
        }
    }

    let score = hp_bar_score(&rgba, WIDTH, HEIGHT, "p1");

    assert!(
        (score - 0.5).abs() < 0.01,
        "隣の画素の成分を混ぜている: {score}"
    );
}
