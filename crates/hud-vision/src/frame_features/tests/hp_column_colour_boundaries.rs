//! HP バーの列ごとの色判定に対するテスト。
//!
//! SF6 は同じバーの上で四つのことを色で伝える。残っている HP（赤／青）、
//! 減った直後の赤み（オレンジ）、危険域（黄）、そして空。判定の境界を
//! 取り違えると、被弾していないのに被弾として数えたり、逆に見落とす。
//!
//! 境界そのものが仕様なので、閾値のすぐ内側と外側の両方を通す。

use super::support::*;
use crate::frame_features::{hp_col_active, hp_col_orange, hp_col_yellow};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

/// ROI の走査帯を指定色で埋めた 1 フレームを作る。
///
/// 上下の除外帯と斜めの傾きは判定側と同じ計算で置く。ここがずれると
/// 「塗ったのに読まれない」だけの無意味なテストになる。
fn frame_filled_with(rgb: (u8, u8, u8)) -> Vec<u8> {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    let (x1, roi_w, y1, roi_h) = (172usize, 681usize, 64usize, 31usize);
    let row_start = 5usize; // HP_COL_ROW_SKIP_TOP
    let row_end = roi_h - 4; // HP_COL_ROW_SKIP_BOTTOM
    for ry in row_start..row_end {
        let x_offset = ((ry - row_start) as f32 * 0.75).round() as usize;
        for cx in 0..roi_w {
            let x = x1 + cx + x_offset;
            if x >= x1 + roi_w {
                continue;
            }
            let index = ((y1 + ry) * WIDTH as usize + x) * 4;
            rgba[index] = rgb.0;
            rgba[index + 1] = rgb.1;
            rgba[index + 2] = rgb.2;
            rgba[index + 3] = 255;
        }
    }
    rgba
}

fn how_many(flags: &[bool]) -> usize {
    flags.iter().filter(|flag| **flag).count()
}

/// ダメージ直後のオレンジは、色相 10〜27 かつ彩度と明度が十分なときだけ。
/// 判定域の内側は拾い、外れた色は拾わない。
#[test]
fn orange_is_recognised_only_inside_its_hue_band() {
    // H≈18, S≈255, V≈220 の橙。
    let inside = hp_col_orange(&frame_filled_with((220, 110, 0)), WIDTH, HEIGHT, "p1");
    assert!(how_many(&inside) > 600, "橙を拾えていない");

    // H≈0 の赤。残 HP の色であって、ダメージ直後の色ではない。
    let red = hp_col_orange(&frame_filled_with((220, 0, 0)), WIDTH, HEIGHT, "p1");
    assert_eq!(how_many(&red), 0, "残 HP の赤を橙と読んでいる");

    // H≈40 の黄緑。橙の帯より外側。
    let beyond = hp_col_orange(&frame_filled_with((200, 220, 0)), WIDTH, HEIGHT, "p1");
    assert_eq!(how_many(&beyond), 0, "帯の外の色を橙と読んでいる");
}

/// 彩度と明度が足りない色は橙と読まない。背景の暗い橙みや、
/// 半透明パネル越しの淡い色を拾うと、被弾が水増しされる。
#[test]
fn a_dull_or_dark_orange_is_not_damage() {
    // 色相は帯の中だが彩度が低い（灰色寄り）。
    let dull = hp_col_orange(&frame_filled_with((150, 130, 120)), WIDTH, HEIGHT, "p1");
    assert_eq!(how_many(&dull), 0, "彩度の低い色を橙と読んでいる");

    // 色相も彩度も帯の中だが暗い。
    let dark = hp_col_orange(&frame_filled_with((60, 30, 0)), WIDTH, HEIGHT, "p1");
    assert_eq!(how_many(&dark), 0, "暗い色を橙と読んでいる");
}

/// 危険域の黄は、橙より高い彩度と明度を要求する。橙と混ざると
/// 「残り少ない」と「いま減った」を取り違える。
#[test]
fn yellow_needs_more_saturation_and_brightness_than_orange() {
    // H≈28, S≈255, V≈255 の黄。
    let yellow = hp_col_yellow(&frame_filled_with((255, 220, 0)), WIDTH, HEIGHT, "p1");
    assert!(how_many(&yellow) > 600, "危険域の黄を拾えていない");

    // 同じ色相でも暗ければ黄ではない。
    let dim = hp_col_yellow(&frame_filled_with((150, 130, 0)), WIDTH, HEIGHT, "p1");
    assert_eq!(how_many(&dim), 0, "暗い黄を危険域と読んでいる");
}

/// 残量の色は側ごとに違う。P1 は赤系、P2 は青系。取り違えると
/// その側の残量が読めなくなる。
#[test]
fn each_side_has_its_own_bar_colour() {
    let red = frame_filled_with((200, 30, 30));
    let blue = frame_filled_with((30, 90, 220));

    assert!(
        how_many(&hp_col_active(&red, WIDTH, HEIGHT, "p1")) > 600,
        "P1 の赤いバーを残量と読めていない"
    );
    assert_eq!(
        how_many(&hp_col_active(&blue, WIDTH, HEIGHT, "p1")),
        0,
        "P1 の判定が青を残量と読んでいる"
    );

    let empty = frame_filled_with((10, 10, 10));
    assert_eq!(
        how_many(&hp_col_active(&empty, WIDTH, HEIGHT, "p1")),
        0,
        "空の帯を残量と読んでいる"
    );
}

/// 危険域の黄は、赤でも青でもないが残量として読む。列のほとんどが
/// 黄でなければ拾わない（髪などのテクスチャを除くため）。
#[test]
fn the_low_health_yellow_still_counts_as_remaining() {
    let yellow = frame_filled_with((255, 220, 0));

    assert!(
        how_many(&hp_col_active(&yellow, WIDTH, HEIGHT, "p1")) > 600,
        "危険域の黄を残量と読めていない"
    );
}

/// 判定は列ごとに独立している。全列が同じ答えになるなら、実際には
/// 列を見ずに一括で決めている。
#[test]
fn each_column_is_judged_on_its_own_pixels() {
    let mut rgba = frame_filled_with((10, 10, 10));
    let (x1, y1) = (172usize, 64usize);
    // 左半分だけを橙にする。斜めの傾きに沿って塗る。
    for ry in 5..27usize {
        let x_offset = ((ry - 5) as f32 * 0.75).round() as usize;
        for cx in 0..300usize {
            let index = ((y1 + ry) * WIDTH as usize + x1 + cx + x_offset) * 4;
            rgba[index] = 220;
            rgba[index + 1] = 110;
            rgba[index + 2] = 0;
        }
    }

    let orange = hp_col_orange(&rgba, WIDTH, HEIGHT, "p1");

    assert!(
        orange[0] && orange[100] && orange[250],
        "塗った側が拾えない"
    );
    assert!(
        !orange[400] && !orange[600],
        "塗っていない側まで橙になっている"
    );
}

/// 左右のバーは傾きが逆向き。側を取り違えると走査位置がずれて、
/// 塗った色を読み落とす。
#[test]
fn the_two_sides_scan_with_opposite_slopes() {
    let painted_for_p1 = frame_filled_with((220, 110, 0));

    let as_p1 = hp_col_orange(&painted_for_p1, WIDTH, HEIGHT, "p1");
    let as_p2 = hp_col_orange(&painted_for_p1, WIDTH, HEIGHT, "p2");

    assert!(how_many(&as_p1) > 600);
    assert_eq!(
        how_many(&as_p2),
        0,
        "P1 側に塗った色を P2 の走査が拾っている"
    );
}

/// 画面全体が入らない短いバッファでも落ちない。デコードの途中で
/// 切れた入力が来る道で、ここで panic すると解析が止まる。
#[test]
fn a_truncated_buffer_reads_nothing_instead_of_panicking() {
    let short = vec![0u8; 1000];

    assert!(hp_col_orange(&short, WIDTH, HEIGHT, "p1")
        .iter()
        .all(|flag| !flag));
    assert!(hp_col_yellow(&short, WIDTH, HEIGHT, "p1")
        .iter()
        .all(|flag| !flag));
    assert!(hp_col_active(&short, WIDTH, HEIGHT, "p1")
        .iter()
        .all(|flag| !flag));
}

/// HUD の帯だけを切り出した入力でも、全画面と同じ答えになる。browser は
/// 帯だけを渡すので、ここがずれると表示と解析が食い違う。
#[test]
fn the_hud_strip_reads_the_same_as_the_whole_frame() {
    let full = frame_filled_with((220, 110, 0));
    let strip = hud_strip_from_frame(&full);

    let from_full = hp_col_orange(&full, WIDTH, HEIGHT, "p1");
    let from_strip =
        crate::frame_features::hp_col_orange_from_hud_strip(&strip, WIDTH, HEIGHT, "p1");

    assert_eq!(from_full, from_strip);
}
