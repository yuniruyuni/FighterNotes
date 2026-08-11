//! デバッグ表示に出す JSON に対するテスト。
//!
//! 読み取りが外れたときに何が起きたのかを追う唯一の窓口なので、ここが
//! 解析本体と食い違うと、直っていないものを直ったと判断する。窓の中身が
//! 本体の読みと一致していることを留める。

use super::support::*;
use crate::frame_features::{hp_bar_debug_json, hp_col_pixel_detail_json};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn parsed(side: &str) -> serde_json::Value {
    let json = hp_bar_debug_json(&make_rgba_p1_bar(0.5), WIDTH, HEIGHT, side);
    serde_json::from_str(&json).expect("JSON である")
}

/// 出す ROI は、実際に読んだ ROI と同じ。ここがずれると、画面上の枠と
/// 数字が別の場所を指す。
#[test]
fn the_reported_roi_is_the_one_that_was_read() {
    let first = parsed("p1");
    let second = parsed("p2");

    assert_eq!(first["roi"]["x1"], 172);
    assert_eq!(first["roi"]["x2"], 853);
    assert_eq!(first["roi"]["y1"], 64);
    assert_eq!(first["roi"]["y2"], 95);
    assert_eq!(second["roi"]["x1"], 1067, "P2 の ROI を出していない");
    assert_eq!(second["roi"]["x2"], 1748);
}

/// 出す残量は、解析が使う残量と同じ値。別々に計算していると、片方だけ
/// 直したときに食い違う。
#[test]
fn the_reported_ratio_matches_what_the_analysis_uses() {
    let rgba = make_rgba_p1_bar(0.5);
    let json = hp_bar_debug_json(&rgba, WIDTH, HEIGHT, "p1");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON である");

    let (analysed, uncertain) = hp_fill_ratio_with_quality(&rgba, WIDTH, HEIGHT, "p1");
    let reported = value["fill_ratio"].as_f64().expect("数値である") as f32;

    assert!(
        (reported - analysed).abs() < 1e-3,
        "表示 {reported} と解析 {analysed} が食い違う"
    );
    assert_eq!(value["uncertain"], uncertain);
}

/// 列の一覧は ROI の幅と同じ長さ。足りなければ表示が縮み、余れば
/// 読んでいない列まで並ぶ。
#[test]
fn every_column_of_the_roi_appears_once() {
    let value = parsed("p1");
    let columns = value["cols"].as_str().expect("文字列である");

    assert_eq!(columns.chars().count(), 681, "列の数が ROI の幅と違う");
}

/// ゾーンは列の一覧を切り分けたもの。隙間なく端まで覆っていること。
#[test]
fn the_zones_cover_the_columns_without_a_gap() {
    let value = parsed("p1");
    let columns: Vec<char> = value["cols"]
        .as_str()
        .expect("文字列である")
        .chars()
        .collect();
    let zones = value["zones"].as_array().expect("配列である");

    assert!(!zones.is_empty(), "ゾーンが一つも無い");
    let mut expected_start = 0u64;
    for zone in zones {
        let start = zone["s"].as_u64().expect("数値である");
        let end = zone["e"].as_u64().expect("数値である");
        let width = zone["w"].as_u64().expect("数値である");

        assert_eq!(start, expected_start, "ゾーンの間に隙間がある");
        assert_eq!(end - start + 1, width, "ゾーンの幅が端と合わない");
        expected_start = end + 1;
    }
    assert_eq!(
        expected_start as usize,
        columns.len(),
        "ゾーンが列の端まで届いていない"
    );
}

/// ゾーンの色名は、その範囲の列の記号と同じものを指す。
#[test]
fn each_zone_names_the_colour_of_its_columns() {
    let value = parsed("p1");
    let columns: Vec<char> = value["cols"]
        .as_str()
        .expect("文字列である")
        .chars()
        .collect();

    for zone in value["zones"].as_array().expect("配列である") {
        let start = zone["s"].as_u64().expect("数値である") as usize;
        let expected = match zone["c"].as_str().expect("文字列である") {
            "White" => 'W',
            "Fill" => 'F',
            "Ghost" => 'G',
            "YW" => 'Y',
            "Orange" => 'O',
            "Dark" => 'D',
            other => panic!("知らない色名: {other}"),
        };

        assert_eq!(columns[start], expected, "{start} 列目の色名が記号と違う");
    }
}

/// 画素ごとの内訳は、頼んだ列だけを返す。
#[test]
fn the_pixel_detail_covers_the_requested_columns() {
    let json = hp_col_pixel_detail_json(&make_rgba_p1_bar(0.5), WIDTH, HEIGHT, "p1", 400, 403);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON である");
    let columns = value.as_array().expect("配列である");

    assert_eq!(columns.len(), 4, "頼んだ列の数と違う");
    for (offset, column) in columns.iter().enumerate() {
        assert_eq!(column["cy"], 400 + offset as u64);
    }
}

/// 画素ごとの内訳が言う列の色は、ゾーン表示の色と同じ。二つが食い違うと、
/// どちらを信じてよいか判らなくなる。
#[test]
fn the_pixel_detail_agrees_with_the_column_summary() {
    let rgba = make_rgba_p1_bar(0.5);
    let summary: serde_json::Value =
        serde_json::from_str(&hp_bar_debug_json(&rgba, WIDTH, HEIGHT, "p1")).expect("JSON である");
    let columns: Vec<char> = summary["cols"]
        .as_str()
        .expect("文字列である")
        .chars()
        .collect();

    let detail: serde_json::Value = serde_json::from_str(&hp_col_pixel_detail_json(
        &rgba, WIDTH, HEIGHT, "p1", 330, 350,
    ))
    .expect("JSON である");

    for column in detail.as_array().expect("配列である") {
        let cy = column["cy"].as_u64().expect("数値である") as usize;
        let named = match column["col_cls"].as_str().expect("文字列である") {
            "White" => 'W',
            "Fill" => 'F',
            "Ghost" => 'G',
            "YW" => 'Y',
            "Orange" => 'O',
            "Dark" => 'D',
            other => panic!("知らない色名: {other}"),
        };

        assert_eq!(columns[cy], named, "{cy} 列目で内訳と要約が食い違う");
    }
}

/// 数えた画素の内訳は、読んだ画素の総数を超えない。
#[test]
fn the_pixel_counts_stay_within_the_pixels_read() {
    let json = hp_col_pixel_detail_json(&make_rgba_p1_bar(0.5), WIDTH, HEIGHT, "p1", 300, 310);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON である");

    for column in value.as_array().expect("配列である") {
        let total = column["total"].as_u64().expect("数値である");
        let rows = column["rows"].as_array().expect("配列である").len() as u64;
        let counted: u64 = ["nW", "nF", "nY", "nO"]
            .iter()
            .map(|key| column[key].as_u64().expect("数値である"))
            .sum();

        assert_eq!(total, rows, "数えた画素と並べた画素が合わない");
        assert!(counted <= total, "内訳の合計が総数を超えている");
        assert!(total > 0, "画素を一つも読んでいない");
    }
}

#[test]
fn pixel_detail_reports_exact_rows_hsv_and_each_counted_colour() {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    let samples = [
        (100usize, [200u8, 200, 200], "White", "nW"),
        (101usize, [220u8, 0, 0], "Fill", "nF"),
        (102usize, [200u8, 180, 150], "YW", "nY"),
        (103usize, [255u8, 190, 0], "Orange", "nO"),
    ];
    for (column, rgb, _, _) in samples {
        for row in HP_COL_ROW_SKIP_TOP..31 - HP_COL_ROW_SKIP_BOTTOM {
            let offset = ((row - HP_COL_ROW_SKIP_TOP) as f32 * HP_BAR_SLOPE).round() as usize;
            let x = 172 + column + offset;
            let y = 64 + row;
            let index = (y * WIDTH as usize + x) * 4;
            rgba[index..index + 3].copy_from_slice(&rgb);
            rgba[index + 3] = 255;
        }
    }

    let detail: serde_json::Value = serde_json::from_str(&hp_col_pixel_detail_json(
        &rgba, WIDTH, HEIGHT, "p1", 100, 103,
    ))
    .expect("JSON である");
    let columns = detail.as_array().expect("配列である");
    assert_eq!(columns.len(), 4);
    for (offset, (column, (_, rgb, class, count_key))) in columns.iter().zip(samples).enumerate() {
        assert_eq!(column["cy"], 100 + offset as u64);
        assert_eq!(column["col_cls"], class);
        assert_eq!(column["total"], 22);
        assert_eq!(column[count_key], 22);
        let rows = column["rows"].as_array().expect("行配列である");
        assert_eq!(rows.len(), 22);
        assert_eq!(rows.first().unwrap()["ry"], 5);
        assert_eq!(rows.last().unwrap()["ry"], 26);
        assert_eq!(rows[0]["r"], rgb[0]);
        assert_eq!(rows[0]["g"], rgb[1]);
        assert_eq!(rows[0]["b"], rgb[2]);
    }
    assert_eq!(columns[1]["rows"][0]["h"], 0);
    assert_eq!(columns[1]["rows"][0]["s"], 255);
    assert_eq!(columns[1]["rows"][0]["v"], 220);
}

#[test]
fn debug_output_serializes_present_and_missing_boundaries() {
    let rgba = make_rgba_p1_bar_yellow_with_orange(0.25, 350, 100);
    let value: serde_json::Value =
        serde_json::from_str(&hp_bar_debug_json(&rgba, WIDTH, HEIGHT, "p1")).expect("JSON である");

    assert!(value["fill_edge_cy"].is_number());
    assert!(value["damage_left_cy"].is_null());
}

/// ROI の外は頼まれても返さない。範囲外を読む手前で止める。
#[test]
fn the_pixel_detail_stops_at_the_edge_of_the_roi() {
    let json = hp_col_pixel_detail_json(&make_rgba_p1_bar(0.5), WIDTH, HEIGHT, "p1", 678, 900);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON である");
    let columns = value.as_array().expect("配列である");

    assert_eq!(columns.len(), 3, "ROI の外まで返している");
    assert_eq!(columns[2]["cy"], 680);
}
