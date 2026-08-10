//! 各読み取りの入口で ROI が成立しているかを見る門と、左右の向きに
//! 対するテスト。
//!
//! ゲージはどれも画面中央側から伸びる。左右で伸びる向きが逆なので、
//! 取り違えると残量が反転する。潰れた ROI から値を出せば、その値は
//! 何も見ていない。

use super::support::*;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

// ── 潰れた ROI ───────────────────────────────────────────────────────────

/// 横だけ、縦だけが潰れた画面でも、値を出さずに諦める。両方潰れた場合
/// しか見ていないと、片方の条件が抜けていても気づけない。
#[test]
fn a_frame_collapsed_on_one_axis_reads_nothing() {
    for (width, height) in [(1u32, HEIGHT), (WIDTH, 1u32), (1, 1)] {
        let rgba = vec![255u8; (width as usize * height as usize * 4).max(4)];

        assert!(
            hp_col_active(&rgba, width, height, "p1").is_empty(),
            "{width}x{height} の画面から列を出している"
        );
        assert_eq!(
            hp_fill_ratio_with_quality(&rgba, width, height, "p1"),
            (0.0, true),
            "{width}x{height} の画面から残量を出している"
        );

        let drive = drive_gauge_read(&rgba, width, height, "left");
        assert!(drive.uncertain, "{width}x{height} の画面から値を出している");
        assert_eq!(drive.value, 0.0);
        assert_eq!(drive.recovery, 0.0);
        assert!(!drive.burnout, "読めていないのにバーンアウトを名乗っている");
    }
}

// ── 左右の向き ───────────────────────────────────────────────────────────

/// 左右の HP バーは画面の別の場所にある。片側に塗った絵で反対側が
/// 埋まってはいけない。
#[test]
fn the_two_health_bars_read_different_places() {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 64..95usize {
        for gx in 172..853usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[220, 30, 30, 255]);
        }
    }

    let first = hp_col_active(&rgba, WIDTH, HEIGHT, "p1");
    let second = hp_col_active(&rgba, WIDTH, HEIGHT, "p2");

    assert!(first[340], "P1 の ROI を読めていない");
    assert!(!second[340], "P1 に塗った絵で P2 が埋まっている");
}

/// ドライブゲージも中央側から伸びる。左右で数え始める端が逆。
#[test]
fn the_drive_gauge_is_read_from_the_centre_outwards() {
    // 左ゲージ ROI の右半分（＝中央側）だけを点灯色で塗る。
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 114..132usize {
        for gx in 723..885usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[240, 210, 40, 255]);
        }
    }

    let from_the_centre = drive_gauge_read(&rgba, WIDTH, HEIGHT, "left");

    assert!(
        !from_the_centre.uncertain,
        "中央側から伸びる光を読めていない"
    );
    assert!(
        (2.5..=3.5).contains(&from_the_centre.value),
        "ROI の半分を塗ったのに {} 本",
        from_the_centre.value
    );
}

/// 右のゲージも同じく中央側から伸びる。ROI の中で数え始める端は
/// 左とは逆になる。
#[test]
fn the_right_drive_gauge_counts_from_its_own_anchor() {
    // 右ゲージ ROI（1036..1360）の左半分＝中央側だけを塗る。
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 114..132usize {
        for gx in 1036..1198usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[240, 210, 40, 255]);
        }
    }

    let from_the_centre = drive_gauge_read(&rgba, WIDTH, HEIGHT, "right");

    assert!(
        !from_the_centre.uncertain,
        "中央側から伸びる光を読めていない"
    );
    assert!(
        (2.5..=3.5).contains(&from_the_centre.value),
        "ROI の半分を塗ったのに {} 本",
        from_the_centre.value
    );
}

/// 右ゲージの外側だけの光もゲージではない。
#[test]
fn light_at_the_far_end_of_the_right_drive_gauge_is_not_a_reading() {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 114..132usize {
        for gx in 1198..1360usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[240, 210, 40, 255]);
        }
    }

    assert!(
        drive_gauge_read(&rgba, WIDTH, HEIGHT, "right").uncertain,
        "外側だけの光から値を出している"
    );
}

/// 反対の端だけが光っているのはゲージではない。向きを取り違えると、
/// 空のゲージが半分溜まって見える。
#[test]
fn light_at_the_far_end_of_the_drive_gauge_is_not_a_reading() {
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for gy in 114..132usize {
        for gx in 561..723usize {
            let index = (gy * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[240, 210, 40, 255]);
        }
    }

    let from_the_far_end = drive_gauge_read(&rgba, WIDTH, HEIGHT, "left");

    assert!(
        from_the_far_end.uncertain,
        "外側だけの光から値を出している: {}",
        from_the_far_end.value
    );
}

// ── 側ごとの判定の緩さ ───────────────────────────────────────────────────

/// 残量とみなすのに必要な列の埋まり具合は左右で違う。P1 側はフレーム
/// メーターのディザリングで抜けるので緩く、P2 側は遮蔽ノイズを落とす
/// ために厳しい。取り違えると、片側だけ残量が痩せるか太る。
#[test]
fn the_two_sides_need_different_amounts_of_a_column() {
    // 走査する 22 行のうち 3 行だけを残量色にする（約 13.6%）。
    // P1 の下限 10% は超え、P2 の下限 15% には届かない。
    let mut rgba = vec![0u8; WIDTH as usize * HEIGHT as usize * 4];
    for row in 0..3usize {
        for gx in 172..853usize {
            let index = ((64 + 5 + row) * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[220, 30, 30, 255]);
        }
    }
    for row in 0..3usize {
        for gx in 1067..1748usize {
            let index = ((64 + 5 + row) * WIDTH as usize + gx) * 4;
            rgba[index..index + 4].copy_from_slice(&[30, 140, 220, 255]);
        }
    }

    let first = hp_col_active(&rgba, WIDTH, HEIGHT, "p1");
    let second = hp_col_active(&rgba, WIDTH, HEIGHT, "p2");

    // 端の列は傾きで一部が ROI の外へ出るため、割合の分母が小さい。
    // 判定の緩さそのものを見たいので、中ほどの列で比べる。
    assert!(first[340], "P1 側の緩い下限が効いていない");
    assert!(!second[340], "P2 側に P1 の緩い下限を使っている");
}

// ── 帯だけを渡す入口 ─────────────────────────────────────────────────────

/// 帯だけを渡しても、全画面と同じ列が黄色と判定される。browser は帯だけを
/// 送るので、ここがずれると解析と手元のデバッグ表示が食い違う。
#[test]
fn the_strip_finds_the_same_yellow_columns_as_the_whole_frame() {
    let full = make_rgba_p1_bar_yellow(0.2);
    let strip = hud_strip_from_frame(&full);

    let from_full = hp_col_yellow(&full, WIDTH, HEIGHT, "p1");
    let from_strip =
        crate::frame_features::hp_col_yellow_from_hud_strip(&strip, WIDTH, HEIGHT, "p1");

    assert!(from_full.iter().any(|value| *value), "黄色を拾えていない");
    assert_eq!(from_full, from_strip, "全画面と帯で判定が食い違う");
}

/// 充填端の位置も返す。デバッグ表示と、どこで減ったかの照合に使う。
#[test]
fn the_decode_reports_where_the_fill_ends() {
    let json =
        crate::frame_features::hp_bar_debug_json(&make_rgba_p1_bar(0.5), WIDTH, HEIGHT, "p1");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON である");

    assert!(
        value["fill_edge_cy"].is_number(),
        "充填端の位置を返していない: {json}"
    );
}
