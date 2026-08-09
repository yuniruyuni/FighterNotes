//! ドライブゲージの読み取り境界に対するテスト。
//!
//! 空き領域は半透明でステージ背景が透けるため、見た目が一定しない。
//! そのため「点灯している列の連鎖」で読み、バーンアウトは EMPTY の文字
//! そのものを手がかりにする。どちらの境界も、外すと残量が丸ごと変わる。

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

/// 途切れなく点灯していれば、その遠端が現在値になる。値は本数で表す
/// （満タンが 6 本）。
#[test]
fn an_unbroken_run_reads_up_to_its_far_end() {
    let read = decode_drive_runs(&runs(&[(Lit, 150)]), COLUMNS);

    assert!(!read.uncertain && !read.burnout);
    assert!(
        (read.value - 3.0).abs() < 0.1,
        "半分まで点いているのに {} 本",
        read.value
    );
}

/// 排出中に分離した小島は、繋いで読む。繋がないと残量が実際より
/// 少なく出る。点いている割合が高いことが条件になる。
#[test]
fn a_detached_island_while_draining_still_continues_the_run() {
    // 62 点灯 → 15 空き → 5 点灯。点いている割合は 0.81。
    let read = decode_drive_runs(&runs(&[(Lit, 62), (Rest, 15), (Lit, 5)]), COLUMNS);

    assert!(!read.uncertain, "分離小島で読み取りを諦めている");
    assert!(
        read.value > 1.3,
        "小島の先まで数えていない: {} 本",
        read.value
    );
}

/// 点いている割合が低ければ、繋がずに読み取りを諦める。まばらな光を
/// 繋ぐと、空のゲージが満タンに見える。
#[test]
fn a_sparse_scatter_of_light_is_not_a_reading() {
    let read = decode_drive_runs(&runs(&[(Lit, 20), (Rest, 100), (Lit, 20)]), COLUMNS);

    assert!(read.uncertain, "まばらな光を繋いで値を出している");
}

/// アンカーの縁が少し欠けていても、そこから始まったものとして読む。
/// 枠線の描画で数ピクセル暗くなることがある。
#[test]
fn a_small_notch_at_the_anchor_does_not_reset_the_reading() {
    let read = decode_drive_runs(&runs(&[(Rest, 5), (Lit, 145)]), COLUMNS);

    assert!(read.value > 0.45, "縁の欠けで読めなくなっている");
}

/// アンカーから離れた場所だけが光っているのは、ゲージではない。
#[test]
fn light_far_from_the_anchor_is_not_the_gauge() {
    let read = decode_drive_runs(&runs(&[(Rest, 100), (Lit, 101)]), COLUMNS);

    assert_eq!(read.value, 0.0, "離れた光を残量と読んでいる");
    assert!(read.uncertain || read.burnout, "静かに 0 を返している");
}

/// 幅のある異物が重なっていたら、読み取りを諦める。キャラクターが
/// ゲージに重なった場面で、当て推量の値を出さないため。
#[test]
fn a_wide_foreign_object_makes_the_reading_uncertain() {
    let read = decode_drive_runs(&runs(&[(Lit, 100), (Foreign, 12), (Lit, 40)]), COLUMNS);

    assert!(read.uncertain, "遮蔽を無視して値を出している");
}

/// ごく細い異物は背景の透けなので、読み取りを止めない。
#[test]
fn a_thin_foreign_sliver_is_tolerated() {
    let read = decode_drive_runs(&runs(&[(Lit, 100), (Foreign, 3), (Lit, 45)]), COLUMNS);

    assert!(!read.uncertain, "背景の透けで読み取りを諦めている");
    assert!(read.value > 2.8, "透けの先まで数えていない: {}", read.value);
}

/// 点灯が皆無でも、アンカーから灰色の帯が伸びていればバーンアウト中の
/// 回復。その遠端が回復の進み具合になる。
#[test]
fn a_grey_slab_from_the_anchor_is_burnout_recovery() {
    let read = decode_drive_runs(&runs(&[(Gray, 150)]), COLUMNS);

    assert!(read.burnout, "回復バーをバーンアウトと読めていない");
    assert!(
        read.recovery > 0.45,
        "回復の進み具合が読めていない: {}",
        read.recovery
    );
}

/// 灰色が細ければ回復バーではない。背景の透けを回復と読むと、
/// バーンアウトしていないのにしていることになる。
#[test]
fn a_thin_grey_line_is_not_a_recovery_bar() {
    let read = decode_drive_runs(&runs(&[(Gray, 6)]), COLUMNS);

    assert!(!read.burnout, "背景の透けを回復バーと読んでいる");
}

/// 点灯も灰色の帯も無ければ、読めなかったものとして扱う。HUD が消えた
/// 場面や全画面フラッシュがこれに当たる。
#[test]
fn nothing_recognisable_is_reported_as_unreadable() {
    let read = decode_drive_runs(&runs(&[]), COLUMNS);

    assert!(read.uncertain, "何も無いのに値を出している");
    assert_eq!(read.value, 0.0);
}
