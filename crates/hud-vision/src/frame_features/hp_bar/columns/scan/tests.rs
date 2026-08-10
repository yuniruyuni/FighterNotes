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

/// 探す色。走査そのものを見たいので、判定が素直に通る一色で塗る。
const WANTED: BarColour = BarColour::DamageOrange;

/// `WANTED` に確実に当たる画素。
const ORANGE: [u8; 3] = [230, 120, 30];

/// 1 画素を塗る。
fn paint(rgba: &mut [u8], x: usize, y: usize) {
    let index = (y * WIDTH as usize + x) * 4;
    rgba[index..index + 3].copy_from_slice(&ORANGE);
    rgba[index + 3] = 255;
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
        paint(&mut rgba, 172 + 100 + x_offset, 64 + ry);
    }

    let hit = scan.columns_where(&rgba, 0.5, WANTED);

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
        paint(&mut rgba, 172 + 100 + x_offset, 64 + ry);
    }

    let as_p1 = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    assert!(as_p1.columns_where(&rgba, 0.5, WANTED)[100]);

    let as_p2 = ColumnScan::new(WIDTH, HEIGHT, "p2", 0).expect("ROI がある");
    assert!(
        !as_p2.columns_where(&rgba, 0.5, WANTED)[100],
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
            paint(&mut rgba, 172 + cx, 64 + ry);
        }
    }

    let hit = scan.columns_where(&rgba, 0.0, WANTED);

    assert!(hit.iter().all(|value| !value), "ふちどりを走査している");
}

/// 割合は「有効だった画素」を分母にする。バッファの外へ出た画素を
/// 分母に含めると、端の列だけ判定が甘くなる。
#[test]
fn the_ratio_counts_only_the_pixels_actually_read() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    // ROI の途中で切れたバッファ。届く範囲はすべて探している色で埋める。
    let mut short = vec![0u8; (64 + 15) * WIDTH as usize * 4];
    for pixel in short.chunks_exact_mut(4) {
        pixel[..3].copy_from_slice(&ORANGE);
        pixel[3] = 255;
    }

    let (matched, effective) = scan.count_in_column(&short, 0, WANTED);

    assert!(effective > 0, "読めた画素まで捨てている");
    assert!(effective < 22, "読めない画素まで分母に入れている");
    assert_eq!(matched, effective, "読めた画素はすべて塗られている");
}

/// 一画素も読めない入力では、どの列も該当しない。0 除算を避ける。
#[test]
fn a_buffer_that_reaches_nothing_matches_no_column() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let empty: Vec<u8> = Vec::new();

    let hit = scan.columns_where(&empty, 0.0, WANTED);

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
            paint(&mut full, 172 + cx + x_offset, 64 + ry);
        }
    }
    let strip_start = (crate::frame_features::HUD_STRIP_Y as f32 * HEIGHT as f32 / 1080.0) as usize;
    let strip_height = crate::frame_features::HUD_STRIP_H as usize;
    let row = WIDTH as usize * 4;
    let strip = full[strip_start * row..(strip_start + strip_height) * row].to_vec();

    let from_full = ColumnScan::new(WIDTH, HEIGHT, "p1", 0)
        .expect("ROI がある")
        .columns_where(&full, 0.5, WANTED);
    let from_strip = ColumnScan::new(WIDTH, HEIGHT, "p1", strip_start)
        .expect("ROI がある")
        .columns_where(&strip, 0.5, WANTED);

    assert_eq!(from_full, from_strip);
}

/// 全面を塗った画で、走査する行数はふちどりを除いた分ちょうど。
/// 一行ずれると、枠線や影が読みに混ざる。
#[test]
fn the_scan_covers_exactly_the_rows_between_the_borders() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let filled = filled_frame();

    let (matched, effective) = scan.count_in_column(&filled, 0, WANTED);

    assert_eq!(effective, 22, "走査する行数が変わっている");
    assert_eq!(matched, effective, "塗った画素を取りこぼしている");
}

/// 行が下がるごとに走査位置が横へずれる。ずれ幅が違うと、平行四辺形の
/// バーを矩形として読むことになり、境界が隣の列とにじむ。
#[test]
fn each_row_is_offset_by_the_bar_slope() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");

    for (row, offset) in [(0usize, 0usize), (1, 1), (10, 8), (21, 16)] {
        let mut rgba = blank();
        paint(&mut rgba, 172 + 100 + offset, 64 + 5 + row);

        let (matched, _) = scan.count_in_column(&rgba, 100, WANTED);

        assert_eq!(matched, 1, "{row} 行目のずれが {offset} 列ではない");
    }
}

/// 傾きで ROI の外へ出た画素は、該当にも分母にも数えない。数えると
/// 端の列だけ判定が甘くなる。
#[test]
fn pixels_pushed_out_of_the_roi_are_not_counted() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let filled = filled_frame();

    let (_, inside) = scan.count_in_column(&filled, 0, WANTED);
    let (_, at_the_edge) = scan.count_in_column(&filled, scan.columns() - 1, WANTED);

    assert_eq!(inside, 22);
    assert_eq!(at_the_edge, 1, "ROI の外へ出た画素まで数えている");
}

/// 探している色以外は数えない。
#[test]
fn a_colour_outside_the_palette_matches_nothing() {
    let scan = ColumnScan::new(WIDTH, HEIGHT, "p1", 0).expect("ROI がある");
    let mut rgba = blank();
    for y in 64..95usize {
        for x in 172..853usize {
            let index = (y * WIDTH as usize + x) * 4;
            rgba[index..index + 4].copy_from_slice(&[30, 60, 230, 255]);
        }
    }

    let (matched, effective) = scan.count_in_column(&rgba, 0, WANTED);

    assert_eq!(effective, 22, "読んだ画素の数が変わっている");
    assert_eq!(matched, 0, "探していない色を数えている");
}

/// 全面を探している色で塗った 1 フレーム。
fn filled_frame() -> Vec<u8> {
    let mut rgba = blank();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[..3].copy_from_slice(&ORANGE);
        pixel[3] = 255;
    }
    rgba
}
