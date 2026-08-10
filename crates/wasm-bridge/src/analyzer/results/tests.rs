//! 解析を始める前の門に対するテスト。
//!
//! ラウンド境界も入力の対応も、間違ったまま進むと以降の全ての判断が
//! ずれる。しかも結果は「それらしい値」になるので、出力からは気づけない。

use super::*;

/// 1 試合は 2 本先取。ラウンド開始演出は 2 回か 3 回出る。
#[test]
fn a_match_shows_the_round_start_two_or_three_times() {
    assert!(marker_count_is_valid(2), "2 本で決着した試合");
    assert!(marker_count_is_valid(3), "3 本目まで行った試合");
}

/// それ以外の数は、動画が途中から始まっているか、中央が隠れているか、
/// 別の何かを拾っている。ラウンド境界には使えない。
#[test]
fn any_other_count_cannot_be_the_round_boundaries() {
    assert!(!marker_count_is_valid(0), "一つも見つかっていない");
    assert!(!marker_count_is_valid(1), "途中から始まる動画");
    assert!(!marker_count_is_valid(4), "誤検出を含んでいる");
}

/// 断った理由には、実際に見つかった数を入れる。数が分からないと、
/// 動画のどこを直せばよいか判断できない。
#[test]
fn the_refusal_says_how_many_were_found() {
    let message = marker_count_error(1);

    assert!(
        message.contains('1'),
        "見つかった数を出していない: {message}"
    );
    assert!(!message.is_empty());
}

/// 入力欄が一つも無ければ使えない。数が合っていても、空同士では
/// 何も対応させられない。
#[test]
fn no_input_rows_at_all_cannot_be_used() {
    let analyzer = Analyzer::new("p1");

    assert!(
        !analyzer.input_rows_are_usable(),
        "空同士を使えることにしている"
    );
}
