//! 入力表示を「一つの入力」の区間へ畳むところに対するテスト。
//!
//! 入力履歴の欄は 1 フレームごとに読む。同じ入力が続いている間は同じ
//! 行が表示され続けるので、そのままでは 1 回の押しが何十件にも見える。
//!
//! 何をもって「同じ入力」とするかは、方向とボタンだけでは足りない。
//! 同じ技を続けて出せば表示も同じになる。表示の継続フレーム数が戻った
//! ことが、新しい入力が始まった印になる。

use super::*;
use crate::input_history::{BadgeColor, BadgeMark};
use match_event_model::test_support::{feat, tracked};

fn screen(count: usize) -> Vec<FrameFeatures> {
    (0..count as u32)
        .map(|frame| feat(frame, 1.0, 1.0))
        .collect()
}

fn punch() -> BadgeMark {
    BadgeMark {
        color: BadgeColor::Red,
        boxed: false,
        glyph: None,
    }
}

/// 継続フレーム数が 1 ずつ増えていく、読めている入力。
fn held(count: u32, dir: InputDir) -> TrackedInput {
    tracked(count, dir, vec![], false, false)
}

fn unreadable() -> TrackedInput {
    TrackedInput {
        count: None,
        dir: InputDir::Unknown,
        badges: vec![],
        auto: false,
        throw: false,
        repaired: false,
        uncertain: true,
    }
}

/// 押しっぱなしは一つの区間。
#[test]
fn a_held_input_is_one_segment() {
    let inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 1);
    assert_eq!((segments[0].start_frame, segments[0].end_frame), (0, 29));
    assert_eq!(segments[0].dir, "R");
}

/// 方向が変われば別の入力。
#[test]
fn a_change_of_direction_starts_a_new_segment() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    for (offset, input) in inputs[15..].iter_mut().enumerate() {
        *input = held(offset as u32 + 1, InputDir::Left);
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].end_frame, 14);
    assert_eq!(segments[1].start_frame, 15);
}

/// 表示が同じままでも、継続フレーム数が戻れば別の入力。同じ技を
/// 二度出したのを一度と数えない。
#[test]
fn the_frame_counter_restarting_starts_a_new_segment() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Neutral)).collect();
    for (offset, input) in inputs[15..].iter_mut().enumerate() {
        *input = held(offset as u32 + 1, InputDir::Neutral);
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 2, "押し直しを一つの入力にしている");
    assert_eq!(segments[1].start_frame, 15);
}

/// 数え直しが無ければ、同じ表示は同じ入力。
#[test]
fn a_rising_frame_counter_keeps_one_segment() {
    let inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Neutral)).collect();

    assert_eq!(build_segments(&screen(30), &inputs).len(), 1);
}

/// ボタンが変われば別の入力。
#[test]
fn a_change_of_button_starts_a_new_segment() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Neutral)).collect();
    for (offset, input) in inputs[15..].iter_mut().enumerate() {
        *input = tracked(
            offset as u32 + 16,
            InputDir::Neutral,
            vec![punch()],
            false,
            false,
        );
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[1].badges, vec!["強".to_string()]);
}

/// 読めなかったフレームで区間を切る。読めていない時間を押しっぱなしと
/// 見なさない。
#[test]
fn an_unreadable_frame_cuts_the_segment() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    for input in &mut inputs[12..18] {
        *input = unreadable();
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 2, "読めない区間をまたいで繋いでいる");
    assert_eq!(segments[0].end_frame, 11);
    assert_eq!(segments[1].start_frame, 18);
}

/// 読めなかったフレームに当たっても、その先の入力を読むのをやめない。
#[test]
fn an_unreadable_frame_does_not_end_the_scan() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    inputs[5] = unreadable();

    let segments = build_segments(&screen(30), &inputs);

    assert!(
        segments.iter().any(|segment| segment.start_frame > 5),
        "読めないフレームで走査を打ち切っている"
    );
}

/// 継続フレーム数が読めていないフレームも区間を切る。
#[test]
fn a_frame_without_a_counter_cuts_the_segment() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    for input in &mut inputs[12..18] {
        input.count = None;
        input.uncertain = false;
    }

    assert_eq!(build_segments(&screen(30), &inputs).len(), 2);
}

/// 試合画面の外は入力欄も無い。
#[test]
fn frames_outside_the_match_screen_cut_the_segment() {
    let inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    let mut features = screen(30);
    for feature in &mut features[12..18] {
        feature.is_match_screen = false;
    }

    assert_eq!(build_segments(&features, &inputs).len(), 2);
}

/// 数え直しは直前のフレームと比べる。区間の先頭と比べると、
/// 数え直した後に伸びていく間ずっと切り続けてしまう。
#[test]
fn the_counter_is_compared_with_the_previous_frame() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 20, InputDir::Neutral)).collect();
    for (offset, input) in inputs[10..].iter_mut().enumerate() {
        *input = held(offset as u32 + 1, InputDir::Neutral);
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 2, "数え直しの後を 1 フレームずつ切っている");
    assert_eq!((segments[1].start_frame, segments[1].end_frame), (10, 29));
}

/// 直接読めたフレームと、トラッカーが補修したフレームを分けて数える。
/// 補修値しか無い区間を「見えていた」と扱わないための材料になる。
#[test]
fn the_segment_counts_observed_and_repaired_frames_apart() {
    let mut inputs: Vec<_> = (0..30).map(|k| held(k + 1, InputDir::Right)).collect();
    for input in &mut inputs[10..20] {
        input.repaired = true;
    }

    let segments = build_segments(&screen(30), &inputs);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].evidence.observed_frames, 20);
    assert_eq!(segments[0].evidence.repaired_frames, 10);
}
