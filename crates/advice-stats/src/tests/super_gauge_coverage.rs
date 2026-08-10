//! SA ゲージの集計を「読めていた」と言えるかの判断に対するテスト。
//!
//! SA の使用回数は、ゲージが減ったことからしか分からない。読めていない
//! 時間があれば、その間に使われた 1 本は数から漏れる。漏れたまま
//! 「使用 0 回」と出すと、使っていないのか読めなかったのか区別できない。
//!
//! だから、ラウンドを通してゲージが読めていたことを別に確かめる。

use super::support::*;
use crate::frame_features::FrameFeatures;
use crate::match_events::RoundInfo;
use crate::temporal::{SUPER_SPEND_CONFIRM_LOOKAHEAD, SUPER_SPEND_CONFIRM_SAMPLES};

const ROUND_FRAMES: u32 = 200;

fn feature(frame_index: u32) -> FrameFeatures {
    FrameFeatures {
        frame_index,
        fps: 60.0,
        own_hp: 1.0,
        opponent_hp: 1.0,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.1,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_super_value: 1.5,
        right_super_value: 2.5,
        left_super_uncertain: false,
        right_super_uncertain: false,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: 1.0,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

/// ゲージが最後まで読めていた 1 ラウンド分の観測。
fn readable_round() -> (Vec<FrameFeatures>, MatchEvents) {
    readable_round_of(ROUND_FRAMES)
}

fn readable_round_of(frames: u32) -> (Vec<FrameFeatures>, MatchEvents) {
    let features = (0..frames).map(feature).collect();
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: frames - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    (features, events)
}

/// 通して読めていれば、SA の集計は完全と言える。
#[test]
fn a_fully_readable_round_makes_the_super_stats_complete() {
    let (features, events) = readable_round();

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert!(stats.super_art_stats_complete, "自分側");
    assert!(stats.opponent_super_art_stats_complete, "相手側");
}

/// 観測そのものが足りないラウンドは、読めていたとは言えない。
#[test]
fn a_round_missing_most_of_its_frames_is_not_complete() {
    let (_, events) = readable_round();
    let sparse: Vec<FrameFeatures> = (0..ROUND_FRAMES).step_by(2).map(feature).collect();

    let stats = build_tactic_stats(&sparse, &events, 1, 2);

    assert!(!stats.super_art_stats_complete);
}

/// 読み取りが怪しいフレームは数に入れない。数だけ揃っていても、
/// 中身が読めていなければ同じこと。
#[test]
fn uncertain_readings_do_not_count_as_coverage() {
    let (mut features, events) = readable_round();
    for feature in &mut features {
        feature.left_super_uncertain = true;
    }

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert!(!stats.super_art_stats_complete, "自分側を完全にしている");
    assert!(
        stats.opponent_super_art_stats_complete,
        "相手側まで巻き添えにしている"
    );
}

/// 試合画面の外は読めていない。
#[test]
fn frames_off_the_match_screen_do_not_count_as_coverage() {
    let (mut features, events) = readable_round();
    for feature in &mut features[20..] {
        feature.is_match_screen = false;
    }

    assert!(!build_tactic_stats(&features, &events, 1, 2).super_art_stats_complete);
}

/// ラウンドの端が読めていなければ完全ではない。端が欠けていると、
/// そこで使われた 1 本を確認しようがない。
#[test]
fn a_round_whose_edges_are_unreadable_is_not_complete() {
    let broken_at = |index: usize| {
        let (mut features, events) = readable_round();
        features[index].left_super_uncertain = true;
        build_tactic_stats(&features, &events, 1, 2).super_art_stats_complete
    };

    assert!(!broken_at(0), "先頭が読めなくても完全にしている");
    assert!(
        !broken_at(ROUND_FRAMES as usize - 1),
        "末尾が読めなくても完全にしている"
    );
    assert!(broken_at(100), "途中の 1 フレームで完全を取り消している");
}

/// 途中に長い読めない区間があれば完全ではない。短い欠測は許す。
///
/// 境目は確定層の窓と同じ。窓の中に必要な標本数が残らなくなる長さから
/// 先は、その間に使われた 1 本を確認できない。
#[test]
fn a_long_blind_stretch_breaks_the_coverage() {
    let blind = |length: usize| {
        let (mut features, events) = readable_round_of(400);
        for feature in &mut features[50..50 + length] {
            feature.left_super_uncertain = true;
        }
        build_tactic_stats(&features, &events, 1, 2).super_art_stats_complete
    };

    assert!(blind(3), "短い欠測で完全を取り消している");
    assert!(
        blind(SUPER_SPEND_CONFIRM_LOOKAHEAD - SUPER_SPEND_CONFIRM_SAMPLES),
        "ちょうどの長さで完全を取り消している"
    );
    assert!(
        !blind(SUPER_SPEND_CONFIRM_LOOKAHEAD - SUPER_SPEND_CONFIRM_SAMPLES + 1),
        "長い欠測を見逃している"
    );
}

/// ラウンドが一つも無ければ、読めていたとは言えない。
#[test]
fn without_any_round_nothing_is_complete() {
    let (features, mut events) = readable_round();
    events.rounds.clear();

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert!(!stats.super_art_stats_complete);
    assert!(!stats.opponent_super_art_stats_complete);
}

// ── ラウンド終了時のゲージ ───────────────────────────────────────────────

/// 終了時のゲージは、最後に試合画面が映っていたフレームから読む。
#[test]
fn the_end_of_round_gauge_comes_from_the_last_match_frame() {
    let (features, events) = readable_round();

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert!((stats.super_gauge_end - 1.5).abs() < 1e-5, "自分側");
    assert!(
        (stats.opponent_super_gauge_end - 2.5).abs() < 1e-5,
        "相手側"
    );
}

/// 自分が右側なら、読む欄も入れ替わる。
#[test]
fn the_gauge_sides_follow_who_the_viewer_is() {
    let (features, events) = readable_round();

    let stats = build_tactic_stats(&features, &events, 2, 1);

    assert!((stats.super_gauge_end - 2.5).abs() < 1e-5, "自分側");
    assert!(
        (stats.opponent_super_gauge_end - 1.5).abs() < 1e-5,
        "相手側"
    );
}

/// ラウンドの外のフレームからは読まない。リザルト画面のゲージは
/// 試合の終了時ではない。
#[test]
fn frames_after_the_round_are_not_the_end_of_round_gauge() {
    let (mut features, events) = readable_round();
    for frame_index in ROUND_FRAMES..ROUND_FRAMES + 60 {
        let mut feature = feature(frame_index);
        feature.left_super_value = 3.0;
        feature.right_super_value = 3.0;
        features.push(feature);
    }

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert!(
        (stats.super_gauge_end - 1.5).abs() < 1e-5,
        "ラウンド外のゲージを読んでいる: {}",
        stats.super_gauge_end
    );
}
