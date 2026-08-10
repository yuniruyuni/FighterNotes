//! 状態の連なりと、テレポートの入力を拾うところに対するテスト。
//!
//! フレームメーターは 1 セルずつ読むので、一つの技も細切れの記録に
//! なる。連なりへまとめ直さないと「何フレーム続いたか」が数えられない。
//!
//! 入力も同じで、ボタンを押し続けている間は同じ表示が並ぶ。どこからが
//! 一つの入力かを決めないと、一度の押しが何度もの押しに見える。

use super::super::*;
use super::support::{teleport_segment, timeline};

fn runs_of(side_runs: &[(u32, u32, &str)], state: &str) -> Vec<StateRun> {
    let timeline = timeline("left", side_runs);
    state_runs(&timeline, |candidate| candidate == state)
}

/// 同じ状態が続くフレームは一つの連なり。
#[test]
fn consecutive_frames_of_one_state_form_a_single_run() {
    let runs = runs_of(&[(10, 19, "active")], "active");

    assert_eq!(runs.len(), 1);
    assert_eq!((runs[0].start, runs[0].end), (10, 19));
    assert_eq!(runs[0].distinct_game_frames, 10);
}

/// 別の状態は数えない。
#[test]
fn other_states_are_not_counted() {
    assert!(runs_of(&[(10, 19, "stun")], "active").is_empty());
}

/// 数フレームの読み落としは同じ連なりとして繋ぐ。読み取りが一瞬
/// こぼれただけで技が二つに割れると、持続の長さが狂う。
#[test]
fn a_short_gap_is_bridged() {
    let bridged = runs_of(&[(10, 19, "active"), (21, 29, "active")], "active");
    let split = runs_of(&[(10, 19, "active"), (22, 29, "active")], "active");

    assert_eq!(bridged.len(), 1, "短い切れ目で割れている");
    assert_eq!((bridged[0].start, bridged[0].end), (10, 29));
    assert_eq!(split.len(), 2, "長い切れ目を繋いでいる");
}

/// 連なりの長さはゲーム内時間で数える。ヒットストップで同じゲーム
/// フレームが何度も映っても、その分は長さに足さない。
#[test]
fn the_length_of_a_run_is_counted_in_game_frames() {
    let stalled = MeterTimeline {
        side: "left".to_string(),
        segments: vec![meter_tracker::TimelineSegment {
            segment_id: 0,
            entries: (0..10)
                .map(|offset| meter_tracker::TimelineEntry {
                    // 同じゲームフレームが 10 動画フレームに伸びている。
                    game_frame: 100,
                    state: "projectile_active".to_string(),
                    video_frame_first: 200 + offset,
                    video_frame_last: 200 + offset,
                    confidence: 1.0,
                })
                .collect(),
        }],
    };

    let runs = state_runs(&stalled, |state| state == "projectile_active");

    assert_eq!(runs.len(), 1);
    assert_eq!((runs[0].start, runs[0].end), (200, 209));
    assert_eq!(
        runs[0].distinct_game_frames, 1,
        "止まっている時間を長さに数えている"
    );
}

/// 動画フレームの分からない記録は使わない。
#[test]
fn entries_without_a_video_frame_are_skipped() {
    let unplaced = MeterTimeline {
        side: "left".to_string(),
        segments: vec![meter_tracker::TimelineSegment {
            segment_id: 0,
            entries: vec![meter_tracker::TimelineEntry {
                game_frame: 5,
                state: "active".to_string(),
                video_frame_first: -1,
                video_frame_last: -1,
                confidence: 1.0,
            }],
        }],
    };

    assert!(state_runs(&unplaced, |state| state == "active").is_empty());
}

// ── テレポートの入力 ─────────────────────────────────────────────────────

/// 無敵の直前にあるボタン同時押しが、テレポートの入力。
#[test]
fn a_button_chord_just_before_the_invincibility_is_the_input() {
    let segments = [teleport_segment(100)];

    let input = teleport_input(&segments, 110).expect("入力が見つかる");

    assert_eq!(input.start_frame, 100);
}

/// 遡る幅には限りがある。ずっと前の同時押しは別の行動。
#[test]
fn a_chord_far_before_the_invincibility_is_a_different_action() {
    let segments = [teleport_segment(100)];

    assert!(teleport_input(&segments, 100 + TELEPORT_INPUT_LOOKBACK).is_some());
    assert!(teleport_input(&segments, 100 + TELEPORT_INPUT_LOOKBACK + 1).is_none());
}

/// 無敵より後ろの入力は原因ではない。表示の遅れ 3 フレームまでは許す。
#[test]
fn a_chord_after_the_invincibility_is_not_the_cause() {
    let segments = [teleport_segment(103)];

    assert!(teleport_input(&segments, 100).is_some());
    assert!(teleport_input(&[teleport_segment(104)], 100).is_none());
}

/// ボタン 1 つの入力はテレポートではない。同時押しが要る。
#[test]
fn a_single_button_is_not_a_teleport_chord() {
    let mut single = teleport_segment(100);
    single.badges = vec!["弱P".to_string()];

    assert!(teleport_input(&[single], 110).is_none());
}

/// パンチとキックが 1 つずつでも同時押しではない。同じ種類が
/// 2 つ以上要る。
#[test]
fn one_punch_and_one_kick_is_not_a_teleport_chord() {
    let mut mixed = teleport_segment(100);
    mixed.badges = vec!["弱P".to_string(), "弱K".to_string()];

    assert!(teleport_input(&[mixed], 110).is_none());
}

/// キックの同時押しでもテレポートになる。
#[test]
fn two_kicks_are_a_teleport_chord() {
    let mut kicks = teleport_segment(100);
    kicks.badges = vec!["弱K".to_string(), "中K".to_string()];

    assert!(teleport_input(&[kicks], 110).is_some());
}

/// 投げや自動入力はテレポートの入力ではない。
#[test]
fn a_throw_or_an_automatic_input_is_not_a_teleport_chord() {
    let mut thrown = teleport_segment(100);
    thrown.throw = true;
    let mut automatic = teleport_segment(100);
    automatic.auto = true;

    assert!(teleport_input(&[thrown], 110).is_none());
    assert!(teleport_input(&[automatic], 110).is_none());
}

/// 押しっぱなしで区間が分かれていても、一つの押しとして先頭を返す。
/// 途中の区間を入力の時刻にすると、実際より遅く押したことになる。
#[test]
fn a_chord_split_across_segments_reports_where_it_started() {
    let segments = [
        teleport_segment(100),
        teleport_segment(104),
        teleport_segment(108),
    ];

    let input = teleport_input(&segments, 112).expect("入力が見つかる");

    assert_eq!(input.start_frame, 100, "押し始めを指していない");
}

/// 間の空いた別の同時押しまでは遡らない。
#[test]
fn an_earlier_separate_chord_is_not_part_of_the_same_press() {
    let segments = [teleport_segment(90), teleport_segment(108)];

    let input = teleport_input(&segments, 112).expect("入力が見つかる");

    assert_eq!(input.start_frame, 108, "別の押しまで繋げている");
}

/// 同時押しがどこにも無ければ、テレポートとは言えない。
#[test]
fn without_any_chord_there_is_no_teleport_input() {
    assert!(teleport_input(&[], 110).is_none());
}
