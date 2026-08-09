//! 列ごと走査に対するテスト。
//!
//! 走査は添字計算そのものなので、ずれても「それらしい答え」が返る。
//! 塗った位置と読まれた位置が一致することを、色の判定から切り離して見る。

use super::*;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn blank() -> Vec<u8> {
    vec![0u8; WIDTH as usize * HEIGHT as usize * 4]
}

/// その画素が塗られているか（黒でないか）だけを見る述語。
fn painted(r: f32, g: f32, b: f32) -> bool {
    r + g + b > 0.0
}

/// ROI の列数は、拡縮後の横幅と一致する。ここがずれると読み取り結果の
/// 長さが変わり、充填率の分母が狂う。
#[test]
fn the_column_count_matches_the_scaled_roi_width() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    assert_eq!(scan.columns(), 681);
}

/// 潰れた画面では走査を作らない。0 除算や範囲外参照の手前で止める。
#[test]
fn a_degenerate_frame_has_no_scan() {
    assert!(ColumnScan::new(0, 0, "p1", 0).is_none());
    assert!(ColumnScan::new(1, 1, "p1", 0).is_none());
}

/// 塗った列だけが該当し、隣は該当しない。列の独立性が崩れていると、
/// 充填の境界が滲む。
#[test]
fn only_the_painted_column_matches() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let mut rgba = blank();
    // 列 100 を、傾きに沿って塗る。
    for ry in 5..27usize {
        let x_offset = ((ry - 5) as f32 * 0.75).round() as usize;
        let index = ((64 + ry) * WIDTH as usize + 172 + 100 + x_offset) * 4;
        rgba[index] = 255;
    }

    let hit = scan.columns_where(&rgba, 0.5, painted);

    assert!(hit[100], "塗った列を拾えていない");
    assert!(!hit[99] && !hit[101], "隣の列まで拾っている");
}

/// 傾きは側で逆向き。P1 の位置に塗った列は、P2 の走査では同じ番号に
/// 現れない。
#[test]
fn the_slope_runs_the_other_way_for_the_second_side() {
    let mut rgba = blank();
    for ry in 5..27usize {
        let x_offset = ((ry - 5) as f32 * 0.75).round() as usize;
        let index = ((64 + ry) * WIDTH as usize + 172 + 100 + x_offset) * 4;
        rgba[index] = 255;
    }

    let as_p1 = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    assert!(as_p1.columns_where(&rgba, 0.5, painted)[100]);

    let as_p2 = ColumnScan::new(WIDTH, HEIGHT, "p2", 0).expect("ROI がある");
    assert!(
        !as_p2.columns_where(&rgba, 0.5, painted)[100],
        "逆向きの走査が同じ列を拾っている"
    );
}

/// 上下のふちどりは走査しない。枠線や影を数えると、空の列が
/// 埋まって見える。
#[test]
fn the_top_and_bottom_borders_are_skipped() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let mut rgba = blank();
    // 除外帯だけを塗る（上 5 行と下 4 行）。
    for ry in (0..5usize).chain(27..31usize) {
        for cx in 0..681usize {
            let index = ((64 + ry) * WIDTH as usize + 172 + cx) * 4;
            rgba[index] = 255;
        }
    }

    let hit = scan.columns_where(&rgba, 0.0, painted);

    assert!(hit.iter().all(|value| !value), "ふちどりを走査している");
}

/// 割合は「有効だった画素」を分母にする。バッファの外へ出た画素を
/// 分母に含めると、端の列だけ判定が甘くなる。
#[test]
fn the_ratio_counts_only_the_pixels_actually_read() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    // ROI の途中で切れたバッファ。
    let short = vec![255u8; (64 + 15) * WIDTH as usize * 4];

    let (matched, effective) = scan.count_in_column(&short, 0, painted);

    assert!(effective > 0, "読めた画素まで捨てている");
    assert!(effective < 22, "読めない画素まで分母に入れている");
    assert_eq!(matched, effective, "読めた画素はすべて塗られている");
}

/// 一画素も読めない入力では、どの列も該当しない。0 除算を避ける。
#[test]
fn a_buffer_that_reaches_nothing_matches_no_column() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let empty: Vec<u8> = Vec::new();

    let hit = scan.columns_where(&empty, 0.0, painted);

    assert_eq!(hit.len(), scan.columns(), "列数は変えない");
    assert!(hit.iter().all(|value| !value));
}

/// 帯だけを切り出した入力でも、全画面と同じ列が該当する。
#[test]
fn a_hud_strip_reads_the_same_columns_as_the_whole_frame() {
    let mut full = blank();
    for ry in 5..27usize {
        let x_offset = ((ry - 5) as f32 * 0.75).round() as usize;
        for cx in 0..200usize {
            let index = ((64 + ry) * WIDTH as usize + 172 + cx + x_offset) * 4;
            full[index] = 255;
        }
    }
    let strip_start = (crate::frame_features::HUD_STRIP_Y as f32 * HEIGHT as f32 / 1080.0) as usize;
    let strip_height = crate::frame_features::HUD_STRIP_H as usize;
    let row = WIDTH as usize * 4;
    let strip = full[strip_start * row..(strip_start + strip_height) * row].to_vec();

    let from_full = ColumnScan::new(WIDTH, HEIGHT, "p1", 0)
        .expect("ROI がある")
        .columns_where(&full, 0.5, painted);
    let from_strip = ColumnScan::new(WIDTH, HEIGHT, "p1", strip_start)
        .expect("ROI がある")
        .columns_where(&strip, 0.5, painted);

    assert_eq!(from_full, from_strip);
}
