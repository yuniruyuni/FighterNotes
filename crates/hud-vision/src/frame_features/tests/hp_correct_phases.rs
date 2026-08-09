//! HP の遡及補正を組み立てる各工程に対するテスト。
//!
//! 1 フレームの読みは上にも下にも外れる。補正は、外れた読みを潰しつつ
//! 本物のダメージを通す、という相反する要求の間に立っている。潰しすぎれば
//! ダメージが記録から消え、通しすぎれば無いダメージが記録に残る。

use crate::frame_features::{
    fill_unreadable_from_the_future, median_smoothed, monotone_forward_pass, reset_round_starts,
    round_segments,
};

// ── 中央値で均す ─────────────────────────────────────────────────────────

/// 1 フレームだけの読み違いは、上下どちらへ外れても消える。
#[test]
fn a_single_frame_misread_is_smoothed_away() {
    let raw = vec![0.50, 0.50, 0.90, 0.50, 0.50, 0.10, 0.50];
    let in_match = vec![true; raw.len()];

    let smoothed = median_smoothed(&raw, &in_match, 2);

    assert_eq!(smoothed, vec![0.50; 7], "1 フレームの外れが残っている");
}

/// 複数フレーム続く変化は本物のダメージ。均しても残る。
#[test]
fn a_sustained_drop_survives_the_smoothing() {
    let raw = vec![0.80, 0.80, 0.80, 0.60, 0.60, 0.60, 0.60];
    let in_match = vec![true; raw.len()];

    let smoothed = median_smoothed(&raw, &in_match, 2);

    assert_eq!(smoothed[0], 0.80, "被弾前が動いている");
    assert_eq!(smoothed[6], 0.60, "被弾後が動いている");
    assert!(smoothed[6] < smoothed[0], "ダメージが消えている");
}

/// 試合外のフレームは窓に入れない。ラウンド間の 0 を混ぜると、
/// 直後のフレームが軒並み低く出る。
#[test]
fn frames_outside_the_match_are_kept_out_of_the_window() {
    let raw = vec![0.0, 0.0, 0.50, 0.50, 0.50];
    let in_match = vec![false, false, true, true, true];

    let smoothed = median_smoothed(&raw, &in_match, 2);

    assert_eq!(smoothed[2], 0.50, "試合外の 0 を窓に入れている");
}

/// 試合外のフレーム自身は書き換えない。
#[test]
fn frames_outside_the_match_are_left_as_they_are() {
    let raw = vec![0.77, 0.50, 0.50];
    let in_match = vec![false, true, true];

    let smoothed = median_smoothed(&raw, &in_match, 2);

    assert_eq!(smoothed[0], 0.77);
}

/// 窓を広げるほど強く均される。幅が効いていなければ、揺れの大きさに
/// 合わせた調整ができない。
#[test]
fn a_wider_window_smooths_more() {
    let raw = vec![0.50, 0.90, 0.90, 0.50, 0.50, 0.50, 0.50];
    let in_match = vec![true; raw.len()];

    let narrow = median_smoothed(&raw, &in_match, 1);
    let wide = median_smoothed(&raw, &in_match, 2);

    assert_eq!(narrow[2], 0.90, "狭い窓で 2 フレームの山を消している");
    assert_eq!(wide[2], 0.50, "広い窓で 2 フレームの山が残っている");
}

// ── ラウンドの切れ目 ──────────────────────────────────────────────────────

/// 試合外を挟んで試合に戻ったところが新しいラウンドの頭。返す並びには
/// 終端も入る。
#[test]
fn a_return_to_the_match_screen_starts_a_new_round() {
    let in_match = vec![true, true, false, false, true, true];

    assert_eq!(round_segments(&in_match), vec![0, 4, 6]);
}

/// 試合画面が途切れなければラウンドは一つ。
#[test]
fn an_uninterrupted_match_is_one_round() {
    assert_eq!(round_segments(&[true; 5]), vec![0, 5]);
}

/// 動画が試合外から始まっても、先頭は常に区間の頭として扱う。
#[test]
fn the_first_frame_always_opens_a_segment() {
    let in_match = vec![false, false, true, true];

    assert_eq!(round_segments(&in_match), vec![0, 2, 4]);
}

/// フレームが無ければ区間も無い。
#[test]
fn no_frames_means_no_rounds() {
    assert_eq!(round_segments(&[]), vec![0, 0]);
}

// ── ラウンドの頭を満タンに戻す ───────────────────────────────────────────

/// ラウンドの頭は必ず満タン。演出でバーが読めず 0.99 になっていても
/// 戻す。
#[test]
fn the_first_frame_of_a_round_is_reset_to_full() {
    let mut corrected = vec![0.30, 0.30, 0.00, 0.99, 0.94];
    let in_match = vec![true, true, false, true, true];

    reset_round_starts(&mut corrected, &in_match);

    assert_eq!(corrected[3], 1.0, "ラウンドの頭を戻していない");
    assert_eq!(corrected[4], 0.94, "頭以外まで書き換えている");
}

/// 動画の先頭から試合が映っている場合は触らない。ラウンド途中からの
/// 録画かもしれず、満タンとは限らない。
#[test]
fn a_recording_that_starts_mid_round_is_left_alone() {
    let mut corrected = vec![0.42, 0.40];
    let in_match = vec![true, true];

    reset_round_starts(&mut corrected, &in_match);

    assert_eq!(corrected[0], 0.42, "途中からの録画を満タンにしている");
}

/// ラウンドの途中の値は動かさない。
#[test]
fn frames_inside_a_round_are_not_reset() {
    let mut corrected = vec![0.90, 0.70, 0.50];
    let in_match = vec![true; 3];

    reset_round_starts(&mut corrected, &in_match);

    assert_eq!(corrected, vec![0.90, 0.70, 0.50]);
}

// ── ラウンド内の単調性 ───────────────────────────────────────────────────

/// ラウンドの中で残量は増えない。増えている読みは直前まで押し下げる。
#[test]
fn health_never_rises_inside_a_round() {
    let mut corrected = vec![0.90, 0.70, 0.85, 0.60];
    let in_match = vec![true; 4];
    let in_uncertain = vec![false; 4];

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &[0, 4]);

    assert_eq!(corrected, vec![0.90, 0.70, 0.70, 0.60]);
}

/// ラウンドをまたげば増えてよい。境目で基準をリセットしないと、
/// 次のラウンドが前のラウンドの残量に頭を抑えられる。
#[test]
fn health_may_rise_across_a_round_boundary() {
    let mut corrected = vec![0.20, 0.10, 1.00, 0.90];
    let in_match = vec![true; 4];
    let in_uncertain = vec![false; 4];

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &[0, 2, 4]);

    assert_eq!(corrected[2], 1.00, "次のラウンドが前の残量に抑えられている");
}

/// 読めなかった上にほぼ 0 のフレームは基準にしない。基準にすると
/// 0 が以降のフレームすべてへ伝わる。
#[test]
fn an_unreadable_zero_does_not_become_the_ceiling() {
    let mut corrected = vec![0.80, 0.00, 0.75];
    let in_match = vec![true; 3];
    let in_uncertain = vec![false, true, false];

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &[0, 3]);

    assert_eq!(corrected[2], 0.75, "消えたバーの 0 が後ろへ伝わっている");
}

/// 読めていれば 0 でも基準になる。KO の 0 まで無視すると、決着の
/// フレームが残量つきで残る。
#[test]
fn a_trusted_zero_is_still_the_ceiling() {
    let mut corrected = vec![0.20, 0.00, 0.05];
    let in_match = vec![true; 3];
    let in_uncertain = vec![false; 3];

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &[0, 3]);

    assert_eq!(corrected[2], 0.00, "KO のあとに残量が戻っている");
}

/// 試合外のフレームは押し下げにも基準にも関わらない。
#[test]
fn frames_outside_the_match_take_no_part_in_the_ceiling() {
    let mut corrected = vec![0.80, 0.30, 0.75];
    let in_match = vec![true, false, true];
    let in_uncertain = vec![false; 3];

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &[0, 3]);

    assert_eq!(corrected[1], 0.30, "試合外を書き換えている");
    assert_eq!(corrected[2], 0.75, "試合外を基準にしている");
}

// ── 読めなかったフレームを埋める ─────────────────────────────────────────

/// 試合外のフレームは、その先で最初に読めた残量で埋める。ラウンド開始
/// 演出の途中に偽の急落を作らないため。
#[test]
fn unreadable_frames_take_the_next_trusted_reading() {
    let mut corrected = vec![0.60, 0.00, 0.00, 1.00];
    let in_match = vec![true, false, false, true];
    let in_uncertain = vec![false; 4];

    fill_unreadable_from_the_future(&mut corrected, &in_match, &in_uncertain);

    assert_eq!(corrected, vec![0.60, 1.00, 1.00, 1.00]);
}

/// 読めなかった上にほぼ 0 のフレームは、試合画面でも埋める対象。
/// 演出でバーが消えているだけで、残量が 0 になったわけではない。
#[test]
fn a_vanished_bar_during_the_match_is_also_filled() {
    let mut corrected = vec![0.60, 0.00, 0.55];
    let in_match = vec![true; 3];
    let in_uncertain = vec![false, true, false];

    fill_unreadable_from_the_future(&mut corrected, &in_match, &in_uncertain);

    assert_eq!(corrected[1], 0.55, "消えたバーを 0 のままにしている");
}

/// 末尾に読めるフレームが無ければ満タンで埋める。試合の終わりが
/// 切れている動画がこれに当たる。
#[test]
fn a_trailing_run_of_unreadable_frames_falls_back_to_full() {
    let mut corrected = vec![0.60, 0.00, 0.00];
    let in_match = vec![true, false, false];
    let in_uncertain = vec![false; 3];

    fill_unreadable_from_the_future(&mut corrected, &in_match, &in_uncertain);

    assert_eq!(corrected[2], 1.0);
}

/// 読めているフレームは動かさない。
#[test]
fn trusted_frames_are_left_as_they_are() {
    let mut corrected = vec![0.90, 0.70, 0.50];
    let in_match = vec![true; 3];
    let in_uncertain = vec![false; 3];

    fill_unreadable_from_the_future(&mut corrected, &in_match, &in_uncertain);

    assert_eq!(corrected, vec![0.90, 0.70, 0.50]);
}
