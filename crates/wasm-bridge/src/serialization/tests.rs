//! browser へ渡す JSON の組み立てに対するテスト。
//!
//! 表示側は、読めなかったことと読めた結果を区別できなければならない。
//! 読めなかった残量を 0 として渡すと、画面には「HP 0%」と出る。

use super::*;
use video_analyzer::{BadgeColor, BadgeMark, InputDir, TrackedInput};

/// Modern 操作の攻撃ボタン一つ。色が強度を表す。
fn badge(color: BadgeColor) -> BadgeMark {
    BadgeMark {
        color,
        boxed: false,
        glyph: None,
    }
}

/// 1 フレーム分の入力表示。
fn input(count: Option<u32>, dir: InputDir, badges: Vec<BadgeMark>) -> TrackedInput {
    TrackedInput {
        count,
        dir,
        badges,
        auto: false,
        throw: false,
        repaired: false,
        uncertain: false,
    }
}

// ── 読めなかった残量 ─────────────────────────────────────────────────────

/// 読めた残量はそのまま渡す。
#[test]
fn a_readable_value_passes_through() {
    assert_eq!(hp_or_unknown(0.42, false), 0.42);
}

/// 読めなかった残量は、あり得ない値で渡す。0 を渡すと、表示側では
/// 瀕死と区別が付かない。
#[test]
fn an_unreadable_value_is_marked_as_out_of_range() {
    let unknown = hp_or_unknown(0.42, true);

    assert!(unknown < 0.0, "読めなかったことを伝えていない: {unknown}");
}

/// 残量が 0 でも、読めていれば 0 のまま渡す。
#[test]
fn a_readable_zero_stays_zero() {
    assert_eq!(hp_or_unknown(0.0, false), 0.0);
}

// ── 入力表示 ─────────────────────────────────────────────────────────────

/// 入力の続いた数、方向、押したボタンを渡す。
#[test]
fn an_input_carries_its_count_direction_and_buttons() {
    let json = tracked_to_json(&[input(
        Some(12),
        InputDir::DownRight,
        vec![badge(BadgeColor::Green)],
    )]);

    assert!(json.contains(r#""count":12"#), "{json}");
    assert!(json.contains(r#""dir":"DR""#), "{json}");
    assert!(json.contains(r#""badges":"#), "{json}");
}

/// 数が読めていなければ、空として渡す。0 を渡すと、押し始めた瞬間と
/// 区別が付かない。
#[test]
fn an_unreadable_count_is_sent_as_empty() {
    let json = tracked_to_json(&[input(None, InputDir::Neutral, vec![])]);

    assert!(json.contains(r#""count":null"#), "{json}");
}

/// 複数のボタンは並べて渡す。
#[test]
fn multiple_buttons_are_listed_together() {
    let json = tracked_to_json(&[input(
        Some(1),
        InputDir::Neutral,
        vec![badge(BadgeColor::Green), badge(BadgeColor::Red)],
    )]);

    let badges = json
        .split(r#""badges":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("バッジの欄がある");
    assert!(badges.contains(' '), "並べていない: {badges}");
}

/// 読み取りの状態もそのまま渡す。表示側で薄く出すために要る。
#[test]
fn the_reading_state_is_carried_through() {
    let mut repaired = input(Some(1), InputDir::Neutral, vec![]);
    repaired.repaired = true;
    repaired.uncertain = true;
    repaired.auto = true;
    repaired.throw = true;

    let json = tracked_to_json(&[repaired]);

    assert!(json.contains(r#""repaired":true"#), "{json}");
    assert!(json.contains(r#""uncertain":true"#), "{json}");
    assert!(json.contains(r#""auto":true"#), "{json}");
    assert!(json.contains(r#""throw":true"#), "{json}");
}

/// フレームは並べて渡す。
#[test]
fn every_frame_appears_in_order() {
    let json = tracked_to_json(&[
        input(Some(1), InputDir::Left, vec![]),
        input(Some(2), InputDir::Right, vec![]),
    ]);

    let left = json.find(r#""dir":"L""#).expect("最初のフレーム");
    let right = json.find(r#""dir":"R""#).expect("次のフレーム");
    assert!(left < right, "順序が入れ替わっている: {json}");
    assert!(json.contains("},{"), "区切っていない: {json}");
}

/// 何も無ければ空。
#[test]
fn no_frames_produce_nothing() {
    assert_eq!(tracked_to_json(&[]), "");
}
