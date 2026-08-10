//! HP バーの色分類に対するテスト。
//!
//! 分類は二段になっている。まず画素ごとにどの色かを決め、次に列の中で
//! その色が占める割合から列の色を決める。
//!
//! 一段目の優先順位を崩すと、残量と減った分の境目が動く。二段目の割合を
//! 崩すと、ディザリングやノイズの乗り方だけで列の色が変わる。

use super::*;

/// 走査が始まる行。
const FIRST_SCANNED_ROW: usize = HP_COL_ROW_SKIP_TOP;
/// 走査される行数。
const SCANNED_ROWS: usize = 22;

// ── 画素ごとの判定 ───────────────────────────────────────────────────────

/// バーの枠は純白。三色とも明るいことを求める。
#[test]
fn a_bright_neutral_pixel_is_the_frame() {
    assert_eq!(
        classify_hp_pixel(200.0, 200.0, 200.0, HpFillHue::Red),
        HpColColor::White
    );
    assert_ne!(
        classify_hp_pixel(180.0, 200.0, 200.0, HpFillHue::Red),
        HpColColor::White,
        "ちょうどの明るさを枠と読んでいる"
    );
}

/// 残量の色は側で違う。P1 は赤、P2 は青。
#[test]
fn the_fill_colour_depends_on_the_side() {
    assert_eq!(
        classify_hp_pixel(220.0, 0.0, 0.0, HpFillHue::Red),
        HpColColor::Fill,
        "P1 の赤"
    );
    assert_eq!(
        classify_hp_pixel(0.0, 140.0, 220.0, HpFillHue::Blue),
        HpColColor::Fill,
        "P2 の青"
    );
    assert_ne!(
        classify_hp_pixel(0.0, 140.0, 220.0, HpFillHue::Red),
        HpColColor::Fill,
        "P1 が青を残量と読んでいる"
    );
    assert_ne!(
        classify_hp_pixel(220.0, 0.0, 0.0, HpFillHue::Blue),
        HpColColor::Fill,
        "P2 が赤を残量と読んでいる"
    );
}

/// 危険域の黄は左右どちらでも残量。バーが黄に変わっても残量は残量。
#[test]
fn the_pinch_yellow_is_fill_on_both_sides() {
    for hue in [HpFillHue::Red, HpFillHue::Blue] {
        assert_eq!(
            classify_hp_pixel(255.0, 238.0, 0.0, hue),
            HpColColor::Fill,
            "危険域の黄を残量と読めていない"
        );
    }
}

/// 同じ黄でも、緑成分が足りなければ残量ではなくダメージの橙。
/// 危険域の黄バーと、その上に乗った高輝度の橙を分ける唯一の手がかり。
#[test]
fn a_yellow_short_of_green_is_damage_not_fill() {
    assert_eq!(
        classify_hp_pixel(255.0, 190.0, 0.0, HpFillHue::Red),
        HpColColor::Orange,
        "橙を危険域の黄と読んでいる"
    );
}

/// コンボで失った分の暗い残像。明るい残量とは明度で分かれる。
#[test]
fn a_dim_saturated_orange_is_the_damage_ghost() {
    assert_eq!(
        classify_hp_pixel(137.0, 122.0, 39.0, HpFillHue::Red),
        HpColColor::Ghost
    );
}

/// 残像より暗い純橙は残像ではない。緑成分の比で分ける。
#[test]
fn a_dim_pure_orange_is_not_the_ghost() {
    assert_ne!(
        classify_hp_pixel(137.0, 100.0, 39.0, HpFillHue::Red),
        HpColColor::Ghost,
        "純橙を残像と読んでいる"
    );
}

/// ダメージ帯の境目に出る明るい黄白。
#[test]
fn a_pale_warm_pixel_is_the_blend_at_the_boundary() {
    assert_eq!(
        classify_hp_pixel(200.0, 180.0, 150.0, HpFillHue::Red),
        HpColColor::YellowWhite
    );
}

/// どれにも当たらなければ空き。半透明の帯から背景が透けるので、
/// 「空きらしい色」があるわけではない。
#[test]
fn anything_else_is_empty() {
    assert_eq!(
        classify_hp_pixel(20.0, 20.0, 30.0, HpFillHue::Red),
        HpColColor::Dark
    );
}

/// 優先順位は上から順。純白は他のどの条件より先に決まる。
#[test]
fn the_frame_wins_over_every_other_colour() {
    // 純白の閾値を超える明るい黄。黄白の条件にも当たるが、枠が優先。
    assert_eq!(
        classify_hp_pixel(255.0, 250.0, 200.0, HpFillHue::Red),
        HpColColor::White
    );
}

// ── 列ごとの判定 ─────────────────────────────────────────────────────────

/// 指定した色を上から順に並べた 1 列分のフレーム。
fn column_of(pixels: &[[u8; 3]]) -> Vec<u8> {
    let mut rgba = vec![0u8; (FIRST_SCANNED_ROW + pixels.len() + HP_COL_ROW_SKIP_BOTTOM) * 4];
    for (offset, colour) in pixels.iter().enumerate() {
        let row = FIRST_SCANNED_ROW + offset;
        rgba[row * 4..row * 4 + 3].copy_from_slice(colour);
        rgba[row * 4 + 3] = 255;
    }
    rgba
}

/// 走査される 22 行のうち、先頭 `count` 行を `colour`、残りを空きにした列。
fn column_with(count: usize, colour: [u8; 3]) -> Vec<u8> {
    let mut pixels = vec![[20u8, 20, 30]; SCANNED_ROWS];
    pixels[..count].fill(colour);
    column_of(&pixels)
}

fn classify(rgba: &[u8]) -> HpColColor {
    let roi = SlantedRoi {
        rgba,
        frame_width: 1,
        x: 0..1,
        y_start: 0,
        height: rgba.len() / 4,
        strip_y: 0,
        slope: 0.0,
    };
    classify_hp_col(&roi, 0, HpFillHue::Red)
}

/// 走査行を 20 行にした列。割合の閾値がちょうどの値で作れる。
fn twenty_rows_with(count: usize, colour: [u8; 3]) -> Vec<u8> {
    let mut pixels = vec![[20u8, 20, 30]; 20];
    pixels[..count].fill(colour);
    column_of(&pixels)
}

const WHITE: [u8; 3] = [200, 200, 200];
const RED: [u8; 3] = [220, 0, 0];
const GHOST: [u8; 3] = [137, 122, 39];
const BLEND: [u8; 3] = [200, 180, 150];
const ORANGE: [u8; 3] = [255, 190, 0];

/// 枠は列の半分を占めていること。少しの白飛びで空きが枠にならないため。
#[test]
fn the_frame_needs_half_the_column() {
    assert_eq!(classify(&column_with(11, WHITE)), HpColColor::White);
    assert_ne!(
        classify(&column_with(10, WHITE)),
        HpColColor::White,
        "半分に満たない白を枠と読んでいる"
    );
}

/// 残量はごく一部でも成立する。フレームメーターの描画で列の大半が
/// 抜けることがあるため。
#[test]
fn a_little_fill_is_enough() {
    assert_eq!(classify(&column_with(3, RED)), HpColColor::Fill);
    assert_ne!(
        classify(&column_with(2, RED)),
        HpColColor::Fill,
        "ノイズ程度の赤を残量と読んでいる"
    );
}

/// 残像は列の四割。少ないと、境目のにじみが残像になる。
#[test]
fn the_ghost_needs_most_of_the_column() {
    assert_eq!(classify(&column_with(9, GHOST)), HpColColor::Ghost);
    assert_ne!(
        classify(&column_with(8, GHOST)),
        HpColColor::Ghost,
        "にじみを残像と読んでいる"
    );
}

/// 境目の黄白も四割。
#[test]
fn the_blend_needs_most_of_the_column() {
    assert_eq!(classify(&column_with(9, BLEND)), HpColColor::YellowWhite);
    assert_ne!(
        classify(&column_with(8, BLEND)),
        HpColColor::YellowWhite,
        "わずかな黄白を境目と読んでいる"
    );
}

/// ダメージの橙は少なめでも成立する。演出の途中でまだらに乗るため。
#[test]
fn the_damage_orange_appears_in_patches() {
    assert_eq!(classify(&column_with(4, ORANGE)), HpColColor::Orange);
    assert_ne!(
        classify(&column_with(3, ORANGE)),
        HpColColor::Orange,
        "まだら未満の橙をダメージと読んでいる"
    );
}

/// 危険域では枠が黄みがかって純白から外れる。白と黄白でほぼ埋まって
/// いて、残量も残像も橙も無ければ、それは枠。
#[test]
fn a_yellowed_frame_is_still_the_frame() {
    let mut pixels = vec![[20u8, 20, 30]; SCANNED_ROWS];
    pixels[..3].fill(WHITE);
    pixels[3..19].fill(BLEND);

    assert_eq!(classify(&column_of(&pixels)), HpColColor::White);
}

/// 残量が混じっていれば、黄みがかった枠とは見なさない。混ぜると、
/// 残量の端が枠として扱われて充填端がずれる。
#[test]
fn a_column_with_any_fill_is_not_a_yellowed_frame() {
    let mut pixels = vec![[20u8, 20, 30]; SCANNED_ROWS];
    pixels[..3].fill(WHITE);
    pixels[3..18].fill(BLEND);
    pixels[18] = RED;

    assert_ne!(classify(&column_of(&pixels)), HpColColor::White);
}

/// 割合はちょうどで足りる。ここを「超える」に読み替えると、境目の
/// 一画素で列の色が変わる。
#[test]
fn the_column_shares_are_met_exactly_at_their_edge() {
    // 走査 20 行のうち、枠は 10 行（半分）、残量は 2 行（一割）、
    // 残像と黄白は 8 行（四割）、橙は 3 行（15%）。
    assert_eq!(classify(&twenty_rows_with(10, WHITE)), HpColColor::White);
    assert_eq!(classify(&twenty_rows_with(2, RED)), HpColColor::Fill);
    assert_eq!(classify(&twenty_rows_with(8, GHOST)), HpColColor::Ghost);
    assert_eq!(
        classify(&twenty_rows_with(8, BLEND)),
        HpColColor::YellowWhite
    );
    assert_eq!(classify(&twenty_rows_with(3, ORANGE)), HpColColor::Orange);
}

/// 黄みがかった枠の条件もちょうどで足りる。白が二割、白と黄白で八割。
#[test]
fn the_yellowed_frame_shares_are_met_exactly_at_their_edge() {
    let mut pixels = vec![[20u8, 20, 30]; 20];
    pixels[..2].fill(WHITE);
    pixels[2..16].fill(BLEND);

    assert_eq!(classify(&column_of(&pixels)), HpColColor::White);
}

/// 一画素も読めない列は空き扱い。0 除算の手前で止める。
#[test]
fn a_column_that_reads_nothing_is_empty() {
    assert_eq!(classify(&[]), HpColColor::Dark);
}

// ── 画素の判定の境目 ─────────────────────────────────────────────────────
//
// 閾値そのものが仕様。三つの channel も、色相・彩度・明度も、
// ちょうどの値とその一つ先の両方を置く。

/// 枠は三色とも明るさを「超えて」いること。一つでも届かなければ枠ではない。
#[test]
fn the_frame_needs_every_channel_past_the_edge() {
    assert_eq!(
        classify_hp_pixel(181.0, 181.0, 181.0, HpFillHue::Red),
        HpColColor::White
    );
    for (channel, rgb) in [
        ("赤", (180.0, 181.0, 181.0)),
        ("緑", (181.0, 180.0, 181.0)),
        ("青", (181.0, 181.0, 180.0)),
    ] {
        assert_ne!(
            classify_hp_pixel(rgb.0, rgb.1, rgb.2, HpFillHue::Red),
            HpColColor::White,
            "{channel}がちょうどでも枠と読んでいる"
        );
    }
}

/// P1 の残量は赤側の色相帯の中だけ。帯の端は含む。
#[test]
fn the_first_players_fill_hue_band_includes_its_edges() {
    assert_eq!(
        classify_hp_pixel(255.0, 170.0, 0.0, HpFillHue::Red),
        HpColColor::Fill,
        "帯の上端"
    );
    assert_ne!(
        classify_hp_pixel(255.0, 178.0, 0.0, HpFillHue::Red),
        HpColColor::Fill,
        "上端の外を残量と読んでいる"
    );
    assert_eq!(
        classify_hp_pixel(213.0, 0.0, 255.0, HpFillHue::Red),
        HpColColor::Fill,
        "巻き戻った側の端"
    );
    assert_ne!(
        classify_hp_pixel(204.0, 0.0, 255.0, HpFillHue::Red),
        HpColColor::Fill,
        "その端の外を残量と読んでいる"
    );
}

/// P1 の残量は彩度と明度を「超えて」いること。ちょうどはスプライトの暗赤。
#[test]
fn the_first_players_fill_needs_saturation_and_brightness_past_the_edge() {
    assert_ne!(
        classify_hp_pixel(204.0, 164.0, 124.0, HpFillHue::Red),
        HpColColor::Fill,
        "ちょうどの彩度を残量と読んでいる"
    );
    assert_ne!(
        classify_hp_pixel(60.0, 0.0, 0.0, HpFillHue::Red),
        HpColColor::Fill,
        "ちょうどの明度を残量と読んでいる"
    );
    assert_eq!(
        classify_hp_pixel(61.0, 0.0, 0.0, HpFillHue::Red),
        HpColColor::Fill,
        "超えた明度を残量と読めていない"
    );
}

/// P2 の残量も同じ。色相帯は青側で、彩度の下限は緩い。
#[test]
fn the_second_players_fill_has_its_own_edges() {
    assert_eq!(
        classify_hp_pixel(0.0, 255.0, 238.0, HpFillHue::Blue),
        HpColColor::Fill,
        "帯の下端"
    );
    assert_ne!(
        classify_hp_pixel(0.0, 255.0, 230.0, HpFillHue::Blue),
        HpColColor::Fill,
        "下端の外"
    );
    assert_eq!(
        classify_hp_pixel(255.0, 0.0, 170.0, HpFillHue::Blue),
        HpColColor::Fill,
        "帯の上端"
    );
    assert_ne!(
        classify_hp_pixel(255.0, 0.0, 162.0, HpFillHue::Blue),
        HpColColor::Fill,
        "上端の外"
    );
    assert_ne!(
        classify_hp_pixel(210.0, 210.0, 255.0, HpFillHue::Blue),
        HpColColor::Fill,
        "ちょうどの彩度"
    );
    assert_ne!(
        classify_hp_pixel(0.0, 0.0, 60.0, HpFillHue::Blue),
        HpColColor::Fill,
        "ちょうどの明度"
    );
}

/// 危険域の黄も、彩度と明度はちょうどでは足りない。
#[test]
fn the_pinch_yellow_needs_saturation_and_brightness_past_the_edge() {
    assert_eq!(
        classify_hp_pixel(255.0, 214.0, 100.0, HpFillHue::Red),
        HpColColor::Fill,
        "危険域の黄を読めていない"
    );
    assert_ne!(
        classify_hp_pixel(204.0, 177.0, 108.0, HpFillHue::Red),
        HpColColor::Fill,
        "ちょうどの彩度を残量と読んでいる"
    );
    assert_ne!(
        classify_hp_pixel(200.0, 168.0, 80.0, HpFillHue::Red),
        HpColColor::Fill,
        "ちょうどの明度を残量と読んでいる"
    );
}

/// 残像は明度の帯の中だけ。明るければ残量、暗ければ空き。
#[test]
fn the_ghost_lives_inside_a_band_of_brightness() {
    assert_eq!(
        classify_hp_pixel(100.0, 89.0, 28.0, HpFillHue::Red),
        HpColColor::Ghost,
        "帯の下端"
    );
    assert_ne!(
        classify_hp_pixel(200.0, 178.0, 56.0, HpFillHue::Red),
        HpColColor::Ghost,
        "帯の上端の外を残像と読んでいる"
    );
}

/// 境目の黄白は三色それぞれの下限を「超えて」いること。
#[test]
fn the_blend_needs_every_channel_past_its_edge() {
    assert_eq!(
        classify_hp_pixel(166.0, 151.0, 101.0, HpFillHue::Red),
        HpColColor::YellowWhite
    );
    for (channel, rgb) in [
        ("赤", (165.0, 151.0, 101.0)),
        ("緑", (166.0, 150.0, 101.0)),
        ("青", (166.0, 151.0, 100.0)),
    ] {
        assert_ne!(
            classify_hp_pixel(rgb.0, rgb.1, rgb.2, HpFillHue::Red),
            HpColColor::YellowWhite,
            "{channel}がちょうどでも境目と読んでいる"
        );
    }
}
