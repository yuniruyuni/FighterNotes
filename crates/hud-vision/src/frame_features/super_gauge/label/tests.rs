//! ラベルの中で文字を拾う条件に対するテスト。
//!
//! ステージの明るい部分もかたまりとして拾える。大きさと位置で絞りきれて
//! いないと、破片の方が数字の位置に近いという理由で選ばれる。

use super::*;

/// ラベルの高さ。実際の値と同じにしておく。
const LABEL_HEIGHT: usize = 75;

/// 指定した大きさのかたまり。
fn component(width: usize, height: usize, area: usize) -> WhiteComponent {
    WhiteComponent {
        x0: 10,
        x1: 10 + width - 1,
        y0: 5,
        y1: 5 + height - 1,
        area,
        seed_x: 10,
        seed_y: 5,
    }
}

/// 数字はラベルの高さのそれなりを占める。背の低い破片は文字ではない。
#[test]
fn a_short_fragment_is_not_a_glyph() {
    let tall = component(20, LABEL_HEIGHT * 2 / 5, 200);
    let short = component(20, LABEL_HEIGHT * 2 / 5 - 1, 200);

    assert!(tall.looks_like_a_glyph(LABEL_HEIGHT), "文字を落としている");
    assert!(
        !short.looks_like_a_glyph(LABEL_HEIGHT),
        "背の低い破片を拾っている"
    );
}

/// 画素数の少ないかたまりも文字ではない。輪郭の縁に出る点がこれに当たる。
#[test]
fn a_tiny_speck_is_not_a_glyph() {
    let solid = component(20, LABEL_HEIGHT, 45);
    let speck = component(20, LABEL_HEIGHT, 44);

    assert!(solid.looks_like_a_glyph(LABEL_HEIGHT), "文字を落としている");
    assert!(!speck.looks_like_a_glyph(LABEL_HEIGHT), "点を拾っている");
}

/// 高さの条件はラベルの高さに比例する。固定値で見ると、縮小率の違う
/// 画面で文字を落とすか破片を拾うかになる。
#[test]
fn the_height_rule_scales_with_the_label() {
    let glyph = component(20, 20, 200);

    assert!(
        glyph.looks_like_a_glyph(50),
        "小さいラベルで文字を落としている"
    );
    assert!(
        !glyph.looks_like_a_glyph(53),
        "大きいラベルで破片を拾っている"
    );
}

/// 数字の位置は左右で鏡像。取り違えると、反対側の何かを数字として読む。
#[test]
fn the_expected_digit_position_mirrors_between_the_sides() {
    let width = 90;
    let left = expected_digit_centre(width, true);
    let right = expected_digit_centre(width, false);

    assert_eq!(left, 72);
    assert_eq!(right, 26);
    assert!(left > width / 2, "左側のゲージの数字が右寄りでない");
    assert!(right < width / 2, "右側のゲージの数字が左寄りでない");
}

/// 位置はラベルの幅に比例する。
#[test]
fn the_expected_digit_position_scales_with_the_label() {
    assert_eq!(expected_digit_centre(180, true), 144);
    assert_eq!(expected_digit_centre(180, false), 52);
}

/// 数字の位置に近いかたまりを選ぶ。ラベル幅の半分を超える幅のものは
/// 数字ではないので候補から外す。
#[test]
fn a_component_wider_than_half_the_label_is_not_the_digit() {
    let digit = WhiteComponent {
        x0: 70,
        x1: 85,
        ..component(16, 60, 400)
    };
    let banner = WhiteComponent {
        x0: 68,
        x1: 88,
        ..component(21, 60, 900)
    };

    let picked = digit_component(&[banner, digit], 40, true);

    assert_eq!(picked.map(|c| c.x0), Some(70), "幅の広い帯を数字にしている");
}

#[test]
fn connected_components_record_seed_width_and_area_exactly() {
    let patch = Patch {
        x: 0,
        y: 0,
        width: 12,
        height: 20,
    };
    let mut rgba = vec![0u8; patch.width * patch.height * 4];
    for y in 2..17 {
        for x in 3..6 {
            let index = (y * patch.width + x) * 4;
            rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }

    let components = white_components(&rgba, patch.width, patch);
    assert_eq!(components.len(), 1);
    let component = components[0];
    assert_eq!((component.x0, component.x1), (3, 5));
    assert_eq!((component.y0, component.y1), (2, 16));
    assert_eq!(component.area, 45);
}

/// 選ぶ位置は側で変わる。同じ二つの候補でも、左のゲージでは右寄りの
/// 方を、右のゲージでは左寄りの方を数字として選ぶ。
#[test]
fn the_side_decides_which_candidate_is_the_digit() {
    let near_the_left = WhiteComponent {
        x0: 20,
        x1: 32,
        ..component(13, 60, 400)
    };
    let near_the_right = WhiteComponent {
        x0: 60,
        x1: 72,
        ..component(13, 60, 400)
    };
    let candidates = [near_the_left, near_the_right];

    let picked_for_the_left = digit_component(&candidates, 90, true);
    let picked_for_the_right = digit_component(&candidates, 90, false);

    assert_eq!(picked_for_the_left.map(|c| c.x0), Some(60), "左側の選び方");
    assert_eq!(picked_for_the_right.map(|c| c.x0), Some(20), "右側の選び方");
}
