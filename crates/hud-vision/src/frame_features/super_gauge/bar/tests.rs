//! SA バーの点灯列から溜まり具合を出す部分に対するテスト。
//!
//! バーの端は中身と関係なく点き、途中は背景が透けて途切れる。どちらの
//! 扱いを外しても溜まりの量がずれ、SA の使用判定が丸ごと動く。

use super::*;

/// 実際のバーの列数。閾値はこの幅を基準に決めてある。
const COLUMNS: usize = 265;

/// 指定した範囲だけが点いた列を作る。範囲は [開始, 終了) の組で、
/// 位置はアンカー側からの通し番号。
fn lit_runs(runs: &[(usize, usize)]) -> Vec<bool> {
    let mut lit = vec![false; COLUMNS];
    for &(start, end) in runs {
        lit[start..end].fill(true);
    }
    lit
}

/// アンカー側から `end` 列目の手前までが点いた列。
fn lit_to(end: usize) -> Vec<bool> {
    lit_runs(&[(0, end)])
}

/// 端から端まで点いていれば満タン。
#[test]
fn a_fully_lit_bar_reads_full() {
    assert_eq!(fraction_from_lit(&lit_to(COLUMNS)), 1.0);
}

/// 一つも点いていなければ空。
#[test]
fn an_unlit_bar_reads_empty() {
    assert_eq!(fraction_from_lit(&lit_runs(&[])), 0.0);
}

/// 半分まで点いていれば、およそ半分になる。
#[test]
fn a_half_lit_bar_reads_about_half() {
    let read = fraction_from_lit(&lit_to(COLUMNS / 2));

    assert!((0.45..=0.55).contains(&read), "割合になっていない: {read}");
}

/// 溜まるほど値が増える。単調でなければ、SA の増減が読み取れない。
#[test]
fn a_longer_run_always_reads_higher() {
    let quarter = fraction_from_lit(&lit_to(COLUMNS / 4));
    let half = fraction_from_lit(&lit_to(COLUMNS / 2));
    let three_quarters = fraction_from_lit(&lit_to(COLUMNS * 3 / 4));

    assert!(
        quarter < half && half < three_quarters,
        "{quarter} / {half} / {three_quarters}"
    );
}

/// 縁取りだけが点いていても溜まりではない。バーの先頭数列は中身と
/// 関係なく点くため、読む範囲から外している。
#[test]
fn light_only_in_the_leading_trim_is_not_a_reading() {
    assert_eq!(fraction_from_lit(&lit_runs(&[(0, 8)])), 0.0);
}

/// 末尾の縁取りを除いても、満タンは満タンとして読める。除いた分だけ
/// 目減りすると、満タンに到達しない。
#[test]
fn the_trailing_trim_does_not_keep_a_full_bar_from_reading_full() {
    assert_eq!(fraction_from_lit(&lit_to(COLUMNS - 10)), 1.0);
}

/// アンカーから少しだけずれて始まる光は、縁の欠けとして受け入れる。
#[test]
fn a_small_notch_at_the_anchor_still_reads() {
    let read = fraction_from_lit(&lit_runs(&[(20, COLUMNS)]));

    assert!(read > 0.9, "縁の欠けで読めなくなっている: {read}");
}

/// アンカーから大きく離れて始まる光はバーではない。背景の色味を
/// 溜まりと読むと、空のゲージが満タンに見える。
#[test]
fn light_far_from_the_anchor_is_not_the_bar() {
    assert_eq!(fraction_from_lit(&lit_runs(&[(21, COLUMNS)])), 0.0);
}

/// 細かい途切れは背景の透け。繋いでその先まで数える。
#[test]
fn a_narrow_gap_is_bridged() {
    let read = fraction_from_lit(&lit_runs(&[(0, 100), (105, 150)]));

    assert!(read > 0.5, "透けの先まで数えていない: {read}");
}

/// 広い途切れの向こうは別物。繋ぐと、溜まっていない分まで数える。
#[test]
fn a_wide_gap_ends_the_reading() {
    let bridged = fraction_from_lit(&lit_runs(&[(0, 100), (105, 150)]));
    let stopped = fraction_from_lit(&lit_runs(&[(0, 100), (106, 150)]));

    assert!(
        stopped < bridged,
        "広い途切れの向こうまで数えている: {stopped} / {bridged}"
    );
    assert!(
        (0.33..=0.42).contains(&stopped),
        "手前で止まっていない: {stopped}"
    );
}

/// 遠く離れた輝きは、いくつあっても読みに入らない。
#[test]
fn a_distant_flash_does_not_extend_the_reading() {
    let alone = fraction_from_lit(&lit_to(100));
    let with_flash = fraction_from_lit(&lit_runs(&[(0, 100), (200, 210)]));

    assert_eq!(alone, with_flash, "離れた輝きを溜まりに数えている");
}

/// 潰れた幅でも範囲外を読まず、割合の範囲に収まった値を返す。
/// 縁取りを外す余地すら無いので、点いた列がそのまま割合になる。
#[test]
fn a_degenerate_width_stays_within_range() {
    assert_eq!(fraction_from_lit(&[]), 0.0);
    assert_eq!(fraction_from_lit(&[false]), 0.0);
    assert_eq!(fraction_from_lit(&[true]), 1.0);
}
