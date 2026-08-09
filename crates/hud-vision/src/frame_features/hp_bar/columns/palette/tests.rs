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
