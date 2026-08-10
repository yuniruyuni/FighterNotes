//! ドライブゲージの連鎖規則の境目に対するテスト。
//!
//! ゲージはアンカー側のセルが最後に減るので、繋がった範囲より先に本物の
//! 点灯は無い。この構造を手がかりに、背景の透けや排出中の小島は繋ぎ、
//! キャラクターの遮蔽やバーンアウト突入の文字は繋がない。
//!
//! どの境目も、片側へずれると別の読みになる。値が出るべきところで
//! 諦めるか、諦めるべきところで当て推量の値が出るかのどちらか。

use super::support::drive_runs_from;
use crate::frame_features::{decode_drive_runs, DriveColClass};
use DriveColClass::*;

/// ROI の全幅。ラン列は端から端まで隙間なく並べる。
const COLUMNS: usize = 300;

/// 幅の並びからラン列を作り、余りを Rest で埋める。
fn runs(spec: &[(DriveColClass, usize)]) -> Vec<(DriveColClass, usize, usize)> {
    let used: usize = spec.iter().map(|&(_, w)| w).sum();
    let mut spec = spec.to_vec();
    if used < COLUMNS {
        spec.push((Rest, COLUMNS - used));
    }
    drive_runs_from(&spec)
}

// ── アンカー側の縁 ───────────────────────────────────────────────────────

/// 縁の欠けは許すが、そこを一列でも超えたらアンカー側が隠れている。
/// 隠れたまま読むと、残っているセルを数え落とす。
#[test]
fn the_anchor_notch_has_a_hard_limit() {
    let notched = decode_drive_runs(&runs(&[(Rest, 8), (Lit, 100)]), COLUMNS);
    let blocked = decode_drive_runs(&runs(&[(Rest, 9), (Lit, 100)]), COLUMNS);

    assert!(!notched.uncertain, "縁の欠けで読めなくなっている");
    assert!(blocked.uncertain, "隠れたアンカーを縁の欠けと読んでいる");
}

// ── 点灯の繋ぎ ───────────────────────────────────────────────────────────

/// 1 セルピッチまでの隙間は排出中の小島。その先まで数える。
#[test]
fn a_gap_of_one_cell_pitch_still_chains() {
    let chained = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 55), (Lit, 3)]), COLUMNS);
    let stopped = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 56), (Lit, 3)]), COLUMNS);

    assert!(chained.value > stopped.value || stopped.uncertain);
    assert!(
        (chained.value - 98.0 / 300.0 * 6.0).abs() < 0.01,
        "小島の先まで数えていない: {}",
        chained.value
    );
}

/// 繋がる範囲の外にある実体のある点灯は、ゲージではない何か。
/// 繋いでしまうと、空のゲージが満タンに見える。
#[test]
fn substantial_light_beyond_the_chain_is_not_the_gauge() {
    let read = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 56), (Lit, 3)]), COLUMNS);

    assert!(read.uncertain, "範囲外の点灯を無視して値を出している");
}

/// 範囲の外にあっても、ごく細い点灯は背景の輝き。読みを止めない。
#[test]
fn a_negligible_speck_beyond_the_chain_is_ignored() {
    let read = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 56), (Lit, 2)]), COLUMNS);

    assert!(!read.uncertain, "背景の輝きで読み取りを諦めている");
    assert!(
        (read.value - 40.0 / 300.0 * 6.0).abs() < 0.01,
        "輝きの先まで数えている: {}",
        read.value
    );
}

/// 大きな隙間の先にある幅広いランは、小島ではなく遮蔽体。
#[test]
fn a_wide_run_after_a_seam_sized_gap_is_an_occluder() {
    let sliver = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 9), (Lit, 24)]), COLUMNS);
    let occluder = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 9), (Lit, 25)]), COLUMNS);

    assert!(!sliver.uncertain, "小島を遮蔽体と読んでいる");
    assert!(occluder.uncertain, "遮蔽体を小島と読んでいる");
}

/// セル間の隙間の内側なら、幅広いランでも遮蔽体ではない。隣り合う
/// セルの継ぎ目がこれに当たる。
#[test]
fn a_wide_run_across_an_ordinary_seam_is_just_the_next_cell() {
    let read = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 8), (Lit, 25)]), COLUMNS);

    assert!(!read.uncertain, "普通の継ぎ目で読み取りを諦めている");
    assert!(
        (read.value - 73.0 / 300.0 * 6.0).abs() < 0.01,
        "継ぎ目の先を数えていない: {}",
        read.value
    );
}

// ── 細切れの点灯をどう読むか ─────────────────────────────────────────────

/// 十分埋まっていれば、実セル幅のランが無くても残量として通す。
#[test]
fn a_well_covered_chain_reads_as_a_value() {
    let read = decode_drive_runs(
        &runs(&[
            (Lit, 20),
            (Rest, 10),
            (Lit, 20),
            (Rest, 10),
            (Lit, 20),
            (Rest, 10),
            (Lit, 10),
        ]),
        COLUMNS,
    );

    assert!(!read.uncertain && !read.burnout);
    assert!(
        (read.value - 100.0 / 300.0 * 6.0).abs() < 0.01,
        "埋まったゲージを読めていない: {}",
        read.value
    );
}

/// 埋まり方が足りない細切れは残量ではない。決まった幅に細い線が
/// 三本以上並ぶのは、バーンアウト突入演出の EMPTY の文字。
#[test]
fn a_sparsely_covered_chain_is_the_burnout_text() {
    let read = decode_drive_runs(
        &runs(&[
            (Lit, 20),
            (Rest, 10),
            (Lit, 20),
            (Rest, 10),
            (Lit, 20),
            (Rest, 11),
            (Lit, 9),
        ]),
        COLUMNS,
    );

    assert!(read.burnout, "EMPTY の文字を残量として読んでいる");
    assert_eq!(read.value, 0.0);
}

/// 実セル幅のランが一本でもあれば、周りが暗くても残量として通す。
/// 排出中は本体セルと小島の間が大きく空く。
#[test]
fn one_real_cell_is_enough_to_read_a_value() {
    let read = decode_drive_runs(&runs(&[(Lit, 40), (Rest, 50), (Lit, 10)]), COLUMNS);

    assert!(!read.uncertain && !read.burnout, "排出中の読みを捨てている");
}

/// 実セルと認める幅には境目がある。狭いストロークの集まりを実セルと
/// 読むと、遮蔽から値が出る。
#[test]
fn the_real_cell_width_has_an_exact_edge() {
    let cell = decode_drive_runs(&runs(&[(Lit, 35), (Rest, 40), (Lit, 10)]), COLUMNS);
    let stroke = decode_drive_runs(&runs(&[(Lit, 34), (Rest, 40), (Lit, 10)]), COLUMNS);

    assert!(!cell.uncertain && !cell.burnout, "実セルを捨てている");
    assert!(stroke.uncertain, "狭いストロークを実セルと読んでいる");
}

/// 埋まり具合は、点いた列を連鎖の長さで割ったもの。長さを一つ取り違えると
/// 境目のちょうどで判定が裏返る。
#[test]
fn the_coverage_is_measured_against_the_whole_chain() {
    let read = decode_drive_runs(
        &runs(&[
            (Lit, 20),
            (Rest, 11),
            (Lit, 20),
            (Rest, 10),
            (Lit, 20),
            (Rest, 10),
            (Lit, 10),
        ]),
        COLUMNS,
    );

    assert!(read.burnout, "連鎖の長さを短く見て残量にしている");
}

/// EMPTY の文字が占める幅より狭い細切れは、文字ではなく遮蔽。
#[test]
fn strokes_narrower_than_the_burnout_text_are_an_occlusion() {
    let text = decode_drive_runs(
        &runs(&[(Lit, 10), (Rest, 25), (Lit, 10), (Rest, 25), (Lit, 11)]),
        COLUMNS,
    );
    let too_narrow = decode_drive_runs(
        &runs(&[(Lit, 10), (Rest, 25), (Lit, 10), (Rest, 25), (Lit, 10)]),
        COLUMNS,
    );

    assert!(text.burnout, "EMPTY の文字を読めていない");
    assert!(too_narrow.uncertain, "狭すぎる断片を文字と読んでいる");
}

/// EMPTY の文字が占める幅より広い細切れも、文字ではなく遮蔽。
#[test]
fn strokes_wider_than_the_burnout_text_are_an_occlusion() {
    let text = decode_drive_runs(
        &runs(&[(Lit, 20), (Rest, 45), (Lit, 20), (Rest, 45), (Lit, 21)]),
        COLUMNS,
    );
    let too_wide = decode_drive_runs(
        &runs(&[(Lit, 20), (Rest, 45), (Lit, 20), (Rest, 45), (Lit, 22)]),
        COLUMNS,
    );

    assert!(text.burnout, "EMPTY の文字を読めていない");
    assert!(too_wide.uncertain, "広すぎる断片を文字と読んでいる");
}

/// 太いストロークは文字ではない。キャラクターの輪郭がこれに当たる。
#[test]
fn a_thick_stroke_is_not_a_letter() {
    let letters = decode_drive_runs(
        &runs(&[(Lit, 24), (Rest, 30), (Lit, 10), (Rest, 30), (Lit, 10)]),
        COLUMNS,
    );
    let outline = decode_drive_runs(
        &runs(&[(Lit, 25), (Rest, 30), (Lit, 10), (Rest, 30), (Lit, 10)]),
        COLUMNS,
    );

    assert!(letters.burnout, "文字を遮蔽と読んでいる");
    assert!(outline.uncertain, "遮蔽を文字と読んでいる");
}

/// 二本しかない細切れは EMPTY ではない。EMPTY は五文字あるので、
/// 二本に見えるのは断片。
#[test]
fn two_strokes_are_too_few_for_the_burnout_text() {
    let read = decode_drive_runs(&runs(&[(Lit, 20), (Rest, 50), (Lit, 20)]), COLUMNS);

    assert!(read.uncertain, "二本の断片を EMPTY と読んでいる");
}

// ── バーンアウト回復バー ─────────────────────────────────────────────────

/// 回復バーもアンカーから始まる。縁の欠けは許すが、離れた灰色は
/// 背景の透け。
#[test]
fn the_recovery_bar_starts_at_the_anchor() {
    let notched = decode_drive_runs(&runs(&[(Rest, 8), (Gray, 20)]), COLUMNS);
    let detached = decode_drive_runs(&runs(&[(Rest, 9), (Gray, 20)]), COLUMNS);

    assert!(notched.burnout, "縁の欠けで回復バーを見失っている");
    assert!(detached.uncertain, "離れた灰色を回復バーと読んでいる");
}

/// 回復バーの中の細かい途切れは描画ノイズ。繋いでその先まで数える。
#[test]
fn a_drawing_notch_inside_the_recovery_bar_is_bridged() {
    let bridged = decode_drive_runs(&runs(&[(Gray, 20), (Rest, 5), (Gray, 19)]), COLUMNS);
    let stopped = decode_drive_runs(&runs(&[(Gray, 20), (Rest, 6), (Gray, 19)]), COLUMNS);

    assert!(bridged.burnout && stopped.burnout);
    assert!(
        bridged.recovery > stopped.recovery,
        "描画ノイズの先まで数えていない: {} / {}",
        bridged.recovery,
        stopped.recovery
    );
}

/// 繋がる範囲の先にある幅のある灰色は、遮蔽体がバーを分断した跡。
/// 分断されたまま読むと、回復の進み具合が実際より小さく出る。
#[test]
fn a_wide_grey_slab_beyond_the_chain_means_the_bar_was_split() {
    let intact = decode_drive_runs(&runs(&[(Gray, 20), (Rest, 10), (Gray, 19)]), COLUMNS);
    let split = decode_drive_runs(&runs(&[(Gray, 20), (Rest, 10), (Gray, 20)]), COLUMNS);

    assert!(!intact.uncertain, "背景の透けで読み取りを諦めている");
    assert!(split.uncertain, "分断されたバーから値を出している");
}

/// 回復バーとして認める最小の幅。これを下回る灰色は背景の透け。
#[test]
fn the_recovery_bar_has_a_minimum_width() {
    // アンカーの縁が欠けた位置から始めることで、幅の測り方も一緒に留める。
    let bar = decode_drive_runs(&runs(&[(Rest, 8), (Gray, 10)]), COLUMNS);
    let bleed = decode_drive_runs(&runs(&[(Rest, 8), (Gray, 9)]), COLUMNS);

    assert!(bar.burnout, "細い回復バーを見失っている");
    assert!(!bleed.burnout, "背景の透けを回復バーと読んでいる");
}

/// 端まで伸びた回復バーは満了。割合が 1.0 を超えないこと。
#[test]
fn a_recovery_bar_across_the_whole_gauge_reads_full() {
    let read = decode_drive_runs(&runs(&[(Gray, COLUMNS)]), COLUMNS);

    assert!(read.burnout);
    assert_eq!(read.recovery, 1.0);
}

/// 回復中でも、幅のある異物が重なっていたら読み取りを諦める。
#[test]
fn a_wide_foreign_object_over_the_recovery_bar_is_an_occlusion() {
    let sliver = decode_drive_runs(&runs(&[(Gray, 30), (Rest, 10), (Foreign, 8)]), COLUMNS);
    let occluder = decode_drive_runs(&runs(&[(Gray, 30), (Rest, 10), (Foreign, 9)]), COLUMNS);

    assert!(!sliver.uncertain, "背景の透けで読み取りを諦めている");
    assert!(occluder.uncertain, "遮蔽を無視して回復量を出している");
}
