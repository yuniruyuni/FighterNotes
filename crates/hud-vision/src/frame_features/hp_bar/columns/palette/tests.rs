//! 画素の色判定に対するテスト。
//!
//! 閾値そのものが仕様なので、帯のすぐ内側と外側の両方を通す。片側だけだと
//! 「広げすぎ」も「狭めすぎ」も見つからない。

use super::*;

/// 橙は色相 10〜27 の帯の中だけ。残 HP の赤（色相 0 付近）と、
/// その先の黄緑を拾ってはいけない。
#[test]
fn damage_orange_lives_between_red_and_yellow() {
    assert!(is_damage_orange(220.0, 110.0, 0.0), "帯の中の橙");
    assert!(!is_damage_orange(220.0, 0.0, 0.0), "残 HP の赤");
    assert!(!is_damage_orange(200.0, 220.0, 0.0), "帯を越えた黄緑");
}

/// 色相が帯の中でも、くすんだ色や暗い色は橙ではない。半透明パネル越しの
/// 背景を拾うと、被弾が水増しされる。
#[test]
fn damage_orange_needs_saturation_and_brightness() {
    assert!(!is_damage_orange(150.0, 130.0, 120.0), "彩度が足りない");
    assert!(!is_damage_orange(60.0, 30.0, 0.0), "明度が足りない");
}

/// 危険域の黄は、橙より狭い色相帯と、高い彩度・明度を要求する。
#[test]
fn low_health_yellow_is_stricter_than_orange() {
    assert!(is_low_health_yellow(255.0, 220.0, 0.0), "危険域の黄");
    assert!(!is_low_health_yellow(150.0, 130.0, 0.0), "暗い黄");
    assert!(!is_low_health_yellow(220.0, 110.0, 0.0), "橙は黄ではない");
}

/// 残量の色は側で違う。P1 は赤系、P2 は青系。
#[test]
fn each_side_has_its_own_remaining_colour() {
    assert!(is_remaining_health("p1", 200.0, 30.0, 30.0), "P1 の赤");
    assert!(
        !is_remaining_health("p1", 30.0, 90.0, 220.0),
        "P1 に青は無い"
    );

    assert!(is_remaining_health("p2", 30.0, 90.0, 220.0), "P2 の青");
    assert!(
        !is_remaining_health("p2", 200.0, 30.0, 30.0),
        "P2 に赤は無い"
    );
}

/// P1 の彩度下限が高いのは、ROI に重なるキャラクターの暗赤を落とすため。
/// ここを緩めると、バーの外側がずっと残量として読まれる。
#[test]
fn a_character_sprite_in_the_roi_is_not_remaining_health() {
    // 彩度 91 のくすんだ赤。スプライトの陰影で、バー本体ではない。
    assert!(!is_remaining_health("p1", 140.0, 90.0, 90.0));
    // 彩度 98.6。閾値のすぐ下は通さない。
    assert!(!is_remaining_health("p1", 150.0, 92.0, 92.0));
    // 彩度 100.3。閾値のすぐ上は通す。
    assert!(is_remaining_health("p1", 150.0, 91.0, 91.0));
    // バー本体は彩度 220 前後で確実に通る。
    assert!(is_remaining_health("p1", 220.0, 20.0, 20.0));
}

/// 空の帯はどの色でもない。
#[test]
fn an_empty_bar_matches_nothing() {
    for (r, g, b) in [(10.0, 10.0, 10.0), (0.0, 0.0, 0.0), (40.0, 40.0, 45.0)] {
        assert!(!is_damage_orange(r, g, b));
        assert!(!is_low_health_yellow(r, g, b));
        assert!(!is_remaining_health("p1", r, g, b));
        assert!(!is_remaining_health("p2", r, g, b));
    }
}

/// 割合の閾値は、それぞれ理由があって違う値になっている。値そのものが
/// 読み取りの厳しさなので、ここで固定する。
#[test]
fn the_ratios_stay_where_they_are() {
    // P1 はフレームメーターのディザリングで列が抜けるぶん緩い。
    assert_eq!(ratio::REMAINING_P1, 0.10);
    // P2 は遮蔽ノイズを許容する。
    assert_eq!(ratio::REMAINING_P2, 0.15);
    // 残量としての黄は、髪などのテクスチャを外すため列のほとんどを求める。
    assert_eq!(ratio::REMAINING_YELLOW, 0.60);
    assert_eq!(ratio::DAMAGE_ORANGE, 0.15);
    assert_eq!(ratio::LOW_HEALTH_YELLOW, 0.15);
}

// ── 名前から判定へ ───────────────────────────────────────────────────────
//
// 走査には探す色を名前で渡す。名前と判定の対応が入れ替わると、橙を
// 探したつもりで黄を数える。

/// 橙の名前は橙の判定に繋がる。
#[test]
fn the_damage_orange_name_selects_the_orange_test() {
    let orange = BarColour::DamageOrange;

    assert!(orange.matches(220.0, 110.0, 0.0), "帯の中の橙");
    assert!(!orange.matches(255.0, 238.0, 0.0), "危険域の黄");
    assert!(!orange.matches(220.0, 0.0, 0.0), "残 HP の赤");
}

/// 黄の名前は黄の判定に繋がる。
#[test]
fn the_low_health_yellow_name_selects_the_yellow_test() {
    let yellow = BarColour::LowHealthYellow;

    assert!(yellow.matches(255.0, 238.0, 0.0), "危険域の黄");
    assert!(!yellow.matches(220.0, 110.0, 0.0), "ダメージの橙");
}

/// 残量の名前は側ごとの判定に繋がる。左右で色相帯が違うので、
/// 取り違えると相手の残量を自分のものとして読む。
#[test]
fn the_remaining_health_name_carries_the_side() {
    let first = BarColour::RemainingHealth { first_player: true };
    let second = BarColour::RemainingHealth {
        first_player: false,
    };
    let red = (220.0, 30.0, 30.0);
    let blue = (30.0, 140.0, 220.0);

    assert!(first.matches(red.0, red.1, red.2), "P1 の赤");
    assert!(
        !first.matches(blue.0, blue.1, blue.2),
        "P1 が青を拾っている"
    );
    assert!(second.matches(blue.0, blue.1, blue.2), "P2 の青");
    assert!(!second.matches(red.0, red.1, red.2), "P2 が赤を拾っている");
}

// ── 閾値の境目 ───────────────────────────────────────────────────────────
//
// 閾値そのものが仕様なので、ちょうどの値とその一つ先の両方を置く。
// 片側だけだと、境目が一段ずれていても気づけない。

/// 橙は彩度を「超えて」いること。ちょうどはくすんだ背景と区別が付かない。
#[test]
fn damage_orange_saturation_must_be_exceeded() {
    assert!(!is_damage_orange(255.0, 225.0, 195.0), "ちょうどの彩度");
    assert!(is_damage_orange(255.0, 224.0, 194.0), "超えた彩度");
}

/// 明度も同じ。暗い橙は半透明パネル越しの背景。
#[test]
fn damage_orange_brightness_must_be_exceeded() {
    assert!(!is_damage_orange(80.0, 70.0, 60.0), "ちょうどの明度");
    assert!(is_damage_orange(81.0, 71.0, 61.0), "超えた明度");
}

/// 危険域の黄は橙より高い彩度を要求する。ここが緩むと、いま減った橙を
/// 「残り少ない」と読む。
#[test]
fn low_health_yellow_saturation_must_be_exceeded() {
    assert!(!is_low_health_yellow(255.0, 247.0, 135.0), "ちょうどの彩度");
    assert!(is_low_health_yellow(255.0, 247.0, 134.0), "超えた彩度");
}

/// 明度も同じ。
#[test]
fn low_health_yellow_brightness_must_be_exceeded() {
    assert!(!is_low_health_yellow(200.0, 193.0, 100.0), "ちょうどの明度");
    assert!(is_low_health_yellow(201.0, 194.0, 101.0), "超えた明度");
}

/// P1 の残量は赤側の色相帯の中だけ。帯の外の橙は残量ではない。
#[test]
fn the_first_players_hue_band_has_exact_edges() {
    assert!(is_remaining_health("p1", 255.0, 170.0, 0.0), "帯の上端");
    assert!(!is_remaining_health("p1", 255.0, 178.0, 0.0), "上端の外");
    assert!(
        is_remaining_health("p1", 213.0, 0.0, 255.0),
        "巻き戻った側の端"
    );
    assert!(!is_remaining_health("p1", 204.0, 0.0, 255.0), "その端の外");
}

/// P1 の彩度下限は、ROI に重なるキャラクターの暗赤を落とすためのもの。
#[test]
fn the_first_players_saturation_must_be_exceeded() {
    assert!(!is_remaining_health("p1", 255.0, 155.0, 155.0), "ちょうど");
    assert!(is_remaining_health("p1", 255.0, 154.0, 154.0), "超えた彩度");
}

/// P1 の明度下限。
#[test]
fn the_first_players_brightness_must_be_exceeded() {
    assert!(!is_remaining_health("p1", 60.0, 0.0, 0.0), "ちょうど");
    assert!(is_remaining_health("p1", 61.0, 0.0, 0.0), "超えた明度");
}

/// P2 の残量は青側の色相帯の中だけ。
#[test]
fn the_second_players_hue_band_has_exact_edges() {
    assert!(is_remaining_health("p2", 0.0, 255.0, 238.0), "帯の下端");
    assert!(!is_remaining_health("p2", 0.0, 255.0, 230.0), "下端の外");
    assert!(is_remaining_health("p2", 255.0, 0.0, 170.0), "帯の上端");
    assert!(!is_remaining_health("p2", 255.0, 0.0, 162.0), "上端の外");
}

/// P2 の彩度下限は P1 より緩い。青いバーは遮蔽で彩度が落ちやすい。
#[test]
fn the_second_players_saturation_must_be_exceeded() {
    assert!(!is_remaining_health("p2", 210.0, 210.0, 255.0), "ちょうど");
    assert!(is_remaining_health("p2", 209.0, 209.0, 255.0), "超えた彩度");
}

/// P2 の明度下限。
#[test]
fn the_second_players_brightness_must_be_exceeded() {
    assert!(!is_remaining_health("p2", 0.0, 0.0, 60.0), "ちょうど");
    assert!(is_remaining_health("p2", 0.0, 0.0, 61.0), "超えた明度");
}

/// 危険域の黄も、彩度はちょうどでは足りない。
#[test]
fn low_health_yellow_saturation_at_the_exact_edge_is_not_enough() {
    assert!(!is_low_health_yellow(204.0, 177.0, 108.0), "ちょうどの彩度");
}

/// P1 の残量も同じ。
#[test]
fn the_first_players_saturation_at_the_exact_edge_is_not_enough() {
    assert!(!is_remaining_health("p1", 204.0, 164.0, 124.0), "ちょうど");
}
