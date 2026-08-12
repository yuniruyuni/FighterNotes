//! バーンアウト中のドライブゲージ読み取りに対するテスト。
//!
//! バーンアウト中のゲージは通常表示と形が違う。二つの食い違いが重なると
//! 回復バーが丸ごと読めなくなる。
//!
//! 1. 回復バーは通常のゲージより細く、下寄りに描かれる。1080p 実測で
//!    通常は y=114〜131 の 18 行、回復バーは y=121〜131 の 11 行で、
//!    さらに上下 y=121,122 と y=131 は V が高い縁になる。灰色の芯は
//!    7 行しかないので、ROI 全高で割ると 7/18 = 0.39 と閾値 0.40 を
//!    ちょうど下回り、列ごとに Gray と Rest が入れ替わって帯が細切れになる。
//! 2. バーは平行四辺形なので、アンカー側の列は下の行ほど ROI の外へ出る。
//!    全行を取れない列はバーではなくその外側を見ており、暗転中は背後の
//!    ステージを読む。点灯中は数行でもセルに当たるので表に出ない。
//!
//! 実動画 2026-08-10 04-52-00.mp4 で、自分側の Drive が確定ラウンドの
//! 21.8% しか読めずバーンアウト表示が確認不能に落ちた場面から起こした。

use super::support::drive_runs_from;
use crate::frame_features::{
    decode_drive_runs, drive_bar_debug_json, drive_gauge_read, DriveColClass, DRIVE_BAR_SLOPE,
    DRIVE_ROI_LEFT, DRIVE_ROI_RIGHT,
};
use DriveColClass::*;

/// 読み取りに使う列数（リム装飾を除いた分）。
const COLUMNS: usize = 314;
/// 傾き 0.625 × 高さ 18 行。アンカーから何列が全行を取れないか。
const TAPER: usize = 11;
/// ROI の行数。
const ROWS: usize = 18;
/// 回復バーが始まる行（y=121）。
const BAR_TOP: usize = 7;
/// 灰白の芯が始まる行（y=123）。上二行は明るい縁。
const CORE_TOP: usize = 9;
/// 灰白の芯が終わる行（y=130）。以降は明るい縁。
const CORE_END: usize = 16;

/// 回復バーの芯（S=0, V=160）。
const CORE: [u8; 4] = [160, 160, 160, 255];
/// 回復バーの縁。彩度は低いが V が高く、芯とは別に見える。
const RIM: [u8; 4] = [235, 238, 240, 255];
/// 暗転したゲージの地（V=30 で Rest）。
const TRACK: [u8; 4] = [12, 16, 30, 255];
/// ゲージの外に見えるステージ背景。高彩度なので単体では Foreign に落ちる。
const STAGE: [u8; 4] = [220, 20, 200, 255];
/// 点灯セル。
const LIT_CELL: [u8; 4] = [240, 200, 0, 255];

/// アンカー起点の列 1 本の、指定した行だけを塗る。index 0 = 画面中央側。
///
/// 座標の作り方は SlantedRoi::column_x と同じで、ROI から出た行は塗らない。
fn paint_rows(
    rgba: &mut [u8],
    side: &str,
    index: usize,
    rows: std::ops::Range<usize>,
    color: [u8; 4],
) {
    let (x1, x2, y1, _) = if side == "left" {
        DRIVE_ROI_LEFT
    } else {
        DRIVE_ROI_RIGHT
    };
    let slope = if side == "left" {
        DRIVE_BAR_SLOPE
    } else {
        -DRIVE_BAR_SLOPE
    };
    // 左ゲージはアンカーが右端なので、列は右から数える。
    let base = if side == "left" {
        x2 as i32 - 1 - index as i32
    } else {
        x1 as i32 + index as i32
    };
    for row in rows {
        let x = base + (row as f32 * slope).round() as i32;
        if x < x1 as i32 || x >= x2 as i32 {
            continue;
        }
        let offset = ((y1 as usize + row) * 1920 + x as usize) * 4;
        rgba[offset..offset + 4].copy_from_slice(&color);
    }
}

/// ROI 全体をステージ背景で埋める。テーパーの列はこれしか映らない。
fn stage_filled_roi(side: &str) -> Vec<u8> {
    let mut rgba = vec![0u8; 1920 * 1080 * 4];
    for index in 0..(DRIVE_ROI_LEFT.1 - DRIVE_ROI_LEFT.0) as usize {
        paint_rows(&mut rgba, side, index, 0..ROWS, STAGE);
    }
    rgba
}

/// バーンアウト回復中のフレームを合成する。回復バーは実測どおり細く、
/// 上下に明るい縁を持つ。
fn burnout_frame(side: &str, recovery_columns: usize) -> Vec<u8> {
    let mut rgba = stage_filled_roi(side);
    for index in TAPER..COLUMNS {
        paint_rows(&mut rgba, side, index, 0..ROWS, TRACK);
        if index < TAPER + recovery_columns {
            paint_rows(&mut rgba, side, index, BAR_TOP..CORE_TOP, RIM);
            paint_rows(&mut rgba, side, index, CORE_TOP..CORE_END, CORE);
            paint_rows(&mut rgba, side, index, CORE_END..ROWS, RIM);
        }
    }
    rgba
}

/// 満タンのゲージ。点灯セルは ROI の全高を埋める。
fn lit_frame(side: &str) -> Vec<u8> {
    let mut rgba = stage_filled_roi(side);
    for index in TAPER..COLUMNS {
        paint_rows(&mut rgba, side, index, 0..ROWS, LIT_CELL);
    }
    rgba
}

/// デバッグ出力の列分類を、デコーダと同じアンカー起点の並びで返す。
/// デバッグ出力は画面順なので、左ゲージは反転してからリムを落とす。
fn cols_of(rgba: &[u8], side: &str) -> String {
    let json = drive_bar_debug_json(rgba, 1920, 1080, side);
    let screen = json
        .split(r#""cols":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("cols");
    let anchored: String = if side == "left" {
        screen.chars().rev().collect()
    } else {
        screen.to_string()
    };
    anchored.chars().take(COLUMNS).collect()
}

// ── 回復バーの細さ ───────────────────────────────────────────────────────

/// 細い回復バーを ROI の全高で測ると、灰色の割合が閾値に届かない。
/// バーが占める行だけで測る。
#[test]
fn the_thin_recovery_bar_is_measured_over_its_own_rows() {
    for side in ["left", "right"] {
        let read = drive_gauge_read(&burnout_frame(side, 120), 1920, 1080, side);

        assert!(read.burnout, "{side}: 細い回復バーを見失っている");
        assert!(!read.uncertain, "{side}: 細い回復バーで諦めている");
        assert!(
            (read.recovery - (TAPER + 120) as f32 / COLUMNS as f32).abs() < 0.02,
            "{side}: 回復の進み具合が合わない: {}",
            read.recovery
        );
    }
}

/// 回復バーの列は途切れず一本の帯になる。ここが列ごとに入れ替わると、
/// 帯が細切れになって分断と区別できなくなる。
#[test]
fn every_column_over_the_recovery_bar_reads_the_same() {
    for side in ["left", "right"] {
        let cols = cols_of(&burnout_frame(side, 120), side);
        let bar: String = cols.chars().skip(TAPER).take(120).collect();

        assert!(
            bar.chars().all(|c| c == 'G'),
            "{side}: 回復バーの列が揃っていない: {bar}"
        );
    }
}

/// 通常の点灯セルは ROI の全高を埋める。バーンアウト側に合わせて
/// 測る行を狭めると、点灯の判定まで緩くなる。
#[test]
fn the_lit_gauge_is_still_measured_over_the_whole_roi() {
    for side in ["left", "right"] {
        let read = drive_gauge_read(&lit_frame(side), 1920, 1080, side);

        assert!(!read.uncertain && !read.burnout, "{side}: 満タンを読めない");
        assert!(
            (read.value - 6.0).abs() < 0.3,
            "{side}: 残量が合わない: {}",
            read.value
        );
    }
}

// ── アンカー端のテーパー ─────────────────────────────────────────────────

/// テーパーの幅は傾きと高さで決まる。ここがずれると、削りすぎて残量の
/// 分解能を落とすか、削り足りずに背景を読む。
#[test]
fn the_taper_covers_exactly_the_sheared_columns() {
    for side in ["left", "right"] {
        let cols = cols_of(&burnout_frame(side, 120), side);
        assert_eq!(
            cols.chars().filter(|&c| c == 'o').count(),
            TAPER,
            "{side}: テーパーの列数が合わない: {cols}"
        );
    }
}

/// テーパーにステージ背景が映っていても、回復バーは読める。
/// 修正前はこの背景が幅 9 列以上の Foreign になり、遮蔽として
/// 読み取り全体を捨てていた。
#[test]
fn a_vivid_stage_behind_the_taper_is_not_an_occlusion() {
    for side in ["left", "right"] {
        let cols = cols_of(&burnout_frame(side, 120), side);
        let taper: String = cols.chars().take(TAPER).collect();

        assert!(
            !taper.contains('F'),
            "{side}: テーパーの背景を遮蔽として読んでいる: {taper}"
        );
    }
}

/// 左右で同じ絵から同じ値が出る。片側だけ座標を間違えても気づけるように。
#[test]
fn both_sides_read_the_same_recovery() {
    let left = drive_gauge_read(&burnout_frame("left", 90), 1920, 1080, "left");
    let right = drive_gauge_read(&burnout_frame("right", 90), 1920, 1080, "right");

    assert_eq!(left.burnout, right.burnout);
    assert!(
        (left.recovery - right.recovery).abs() < 0.01,
        "左右で回復量が違う: {} / {}",
        left.recovery,
        right.recovery
    );
}

// ── ラン列から見た振る舞い ───────────────────────────────────────────────

/// 幅の並びからラン列を作り、余りを Rest で埋める。
fn runs(spec: &[(DriveColClass, usize)]) -> Vec<(DriveColClass, usize, usize)> {
    let used: usize = spec.iter().map(|&(_, w)| w).sum();
    let mut spec = spec.to_vec();
    if used < COLUMNS {
        spec.push((Rest, COLUMNS - used));
    }
    drive_runs_from(&spec)
}

/// 縁の欠けを許す幅は、測れている最初の列から数える。テーパーの分まで
/// 縁の欠けに使うと、隠れたアンカーを見逃す。
#[test]
fn the_anchor_notch_is_measured_from_the_first_real_column() {
    let notched = decode_drive_runs(&runs(&[(Outside, TAPER), (Rest, 8), (Lit, 100)]), COLUMNS);
    let blocked = decode_drive_runs(&runs(&[(Outside, TAPER), (Rest, 9), (Lit, 100)]), COLUMNS);

    assert!(!notched.uncertain, "縁の欠けで読めなくなっている");
    assert!(blocked.uncertain, "隠れたアンカーを縁の欠けと読んでいる");
}

/// 測れていない列は埋まり具合の分母から外す。数えると、満タンのゲージが
/// まばらな点灯に見える。
#[test]
fn the_taper_is_not_counted_against_the_lit_coverage() {
    let read = decode_drive_runs(&runs(&[(Outside, TAPER), (Lit, 100)]), COLUMNS);

    assert!(!read.uncertain && !read.burnout);
    assert!(
        (read.value - (TAPER + 100) as f32 / COLUMNS as f32 * 6.0).abs() < 0.01,
        "残量が合わない: {}",
        read.value
    );
}
