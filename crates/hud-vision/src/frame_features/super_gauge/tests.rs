use super::*;

const WIDTH: usize = 1920;
const HEIGHT: usize = 70;

#[test]
fn reads_mirrored_partial_gauges_and_integer_labels() {
    for (side, level, fraction) in [
        ("left", 0, 0.42),
        ("left", 2, 0.68),
        ("right", 1, 0.25),
        ("right", 3, 1.0),
    ] {
        let mut rgba = vec![0; WIDTH * HEIGHT * 4];
        paint_gauge(&mut rgba, side, level, fraction);
        let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, side);
        let expected = if level == 3 {
            3.0
        } else {
            level as f32 + fraction
        };
        assert_eq!(read.displayed_level, Some(level));
        assert!(!read.uncertain);
        assert!(
            (read.value - expected).abs() < 0.08,
            "{side} level={level} expected={expected} actual={}",
            read.value
        );
    }
}

#[test]
fn missing_label_is_uncertain_even_if_stage_color_resembles_the_bar() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    let (x, y, width, height) = PACKED_BAR_LEFT;
    fill_rect(
        &mut rgba,
        x + width / 2,
        y,
        width / 3,
        height,
        [220, 20, 140],
    );
    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "left");
    assert!(read.uncertain);
    assert_eq!(read.displayed_level, None);
    assert_eq!(read.value, 0.0);
}

#[test]
fn three_with_leftward_serifs_is_not_mistaken_for_zero() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    paint_gauge(&mut rgba, "right", 3, 1.0);
    let (label_x, label_y, _, label_height) = PACKED_LABEL_RIGHT;
    let digit_x = label_x + 11;
    let digit_y = label_y + 8;
    let digit_height = label_height - 12;
    let white = [245, 245, 245];

    // 実際の描き文字の 3 は上下端が左へ張り出す。旧来の矩形塗り率では
    // この二つを 0 の左縦線と誤認したが、中央には外へ開いた隙間がある。
    fill_rect(&mut rgba, digit_x, digit_y + 6, 5, 14, white);
    fill_rect(
        &mut rgba,
        digit_x,
        digit_y + 40,
        5,
        digit_height - 40,
        white,
    );

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "right");
    assert_eq!(read.displayed_level, Some(3));
    assert_eq!(read.value, 3.0);
}

#[test]
fn detached_bright_background_does_not_close_the_three_glyph() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    paint_gauge(&mut rgba, "right", 3, 1.0);
    let (label_x, label_y, _, _) = PACKED_LABEL_RIGHT;
    let digit_x = label_x + 11;
    let digit_y = label_y + 8;

    // 3 の上側開口内にある明るい背景。グリフとは連結していないが、
    // bbox 内の全白画素を混ぜると膨張後に開口を塞いで 0 に見える。
    fill_rect(&mut rgba, digit_x + 1, digit_y + 8, 4, 17, [245, 245, 245]);

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "right");
    assert_eq!(read.displayed_level, Some(3));
}

#[test]
fn digit_and_detached_bright_component_are_not_ca() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    paint_gauge(&mut rgba, "right", 2, 0.4);
    let (label_x, label_y, _, _) = PACKED_LABEL_RIGHT;
    fill_rect(
        &mut rgba,
        label_x + 55,
        label_y + 8,
        15,
        50,
        [245, 245, 245],
    );

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "right");
    assert!(!read.critical_art);
    assert_eq!(read.displayed_level, Some(2));
}

#[test]
fn ca_requires_the_c_and_a_hole_topology() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    let (label_x, label_y, _, _) = PACKED_LABEL_RIGHT;
    let white = [245, 245, 245];
    let y = label_y + 8;
    let height = 52;
    let width = 25;
    let thickness = 6;

    // C
    fill_rect(&mut rgba, label_x + 4, y, width, thickness, white);
    fill_rect(
        &mut rgba,
        label_x + 4,
        y + height - thickness,
        width,
        thickness,
        white,
    );
    fill_rect(&mut rgba, label_x + 4, y, thickness, height, white);
    // A（上半分に閉じた穴を持つ）
    let a_x = label_x + 35;
    fill_rect(&mut rgba, a_x, y, width, thickness, white);
    fill_rect(&mut rgba, a_x, y, thickness, height, white);
    fill_rect(
        &mut rgba,
        a_x + width - thickness,
        y,
        thickness,
        height,
        white,
    );
    fill_rect(&mut rgba, a_x, y + height / 2, width, thickness, white);

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "right");
    assert!(read.critical_art);
    assert_eq!(read.displayed_level, Some(3));
    assert_eq!(read.value, 3.0);
}

/// ゲージの色は側ごとに違う。左は桃、右は水色。取り違えると、その側の
/// 溜まり具合が読めない。
#[test]
fn each_side_reads_only_its_own_bar_colour() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    paint_gauge(&mut rgba, "left", 1, 0.60);
    // 左の帯の位置へ、右側の色を塗り足す。左の判定は拾ってはいけない。
    let (bx, by, bw, bh) = PACKED_BAR_RIGHT;
    fill_rect(&mut rgba, bx + 8, by + 4, bw / 2, bh - 8, [230, 25, 145]);

    let right = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "right");

    assert_eq!(right.value, 0.0, "右の判定が左の色を拾っている");
}

/// 帯の端にある枠や角の光沢は、溜まりとして数えない。数えると
/// 空のゲージが少し溜まって見える。
#[test]
fn the_padding_at_both_ends_is_not_part_of_the_fill() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    let (lx, ly, _, lh) = PACKED_LABEL_LEFT;
    paint_digit(&mut rgba, lx + 50, ly + 8, 30, lh - 12, 0);
    let (bx, by, bw, bh) = PACKED_BAR_LEFT;
    // 左右の端だけを光らせる。可視の帯そのものではない。
    fill_rect(&mut rgba, bx, by + 4, 6, bh - 8, [230, 25, 145]);
    fill_rect(&mut rgba, bx + bw - 6, by + 4, 6, bh - 8, [230, 25, 145]);

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "left");

    assert_eq!(read.value, 0.0, "端の光を溜まりと読んでいる");
}

/// 帯の途中が大きく途切れていたら、その先は溜まりに数えない。演出や
/// 遮蔽で飛び地が光ることがあり、それを繋ぐと過大に読む。
#[test]
fn a_wide_gap_stops_the_fill_from_continuing() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    let (lx, ly, _, lh) = PACKED_LABEL_LEFT;
    paint_digit(&mut rgba, lx + 50, ly + 8, 30, lh - 12, 1);
    let (bx, by, bw, bh) = PACKED_BAR_LEFT;
    let usable = bw - 18;
    // 手前 20% を光らせ、大きく空けてから終端付近をもう一度光らせる。
    fill_rect(
        &mut rgba,
        bx + 8,
        by + 4,
        usable / 5,
        bh - 8,
        [230, 25, 145],
    );
    fill_rect(
        &mut rgba,
        bx + 8 + usable * 4 / 5,
        by + 4,
        usable / 5,
        bh - 8,
        [230, 25, 145],
    );

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "left");

    assert!(
        read.value - 1.0 < 0.45,
        "飛び地を繋いで読んでいる: {}",
        read.value
    );
}

/// 溜まりが帯の手前から始まっていなければ、読み取りを諦める。途中から
/// 光っているのはゲージではなく、背景か演出である。
#[test]
fn a_fill_that_does_not_start_at_the_near_end_is_rejected() {
    let mut rgba = vec![0; WIDTH * HEIGHT * 4];
    let (lx, ly, _, lh) = PACKED_LABEL_LEFT;
    paint_digit(&mut rgba, lx + 50, ly + 8, 30, lh - 12, 2);
    let (bx, by, bw, bh) = PACKED_BAR_LEFT;
    // 帯の真ん中だけが光っている。
    fill_rect(
        &mut rgba,
        bx + bw / 2,
        by + 4,
        bw / 4,
        bh - 8,
        [230, 25, 145],
    );

    let read = super_gauge_read_from_hud_strip(&rgba, WIDTH as u32, "left");

    assert_eq!(read.value, 2.0, "途中から光る帯を溜まりと読んでいる");
}

fn paint_gauge(rgba: &mut [u8], side: &str, level: u8, fraction: f32) {
    let is_left = side == "left";
    let (lx, ly, lw, lh) = if is_left {
        PACKED_LABEL_LEFT
    } else {
        PACKED_LABEL_RIGHT
    };
    let digit_x = if is_left { lx + 50 } else { lx + 11 };
    paint_digit(rgba, digit_x, ly + 8, 30, lh - 12, level);

    let (bx, by, bw, bh) = if is_left {
        PACKED_BAR_LEFT
    } else {
        PACKED_BAR_RIGHT
    };
    let pad = 8;
    let usable = bw - 18;
    let lit_width = (usable as f32 * fraction) as usize;
    let color = if is_left {
        [230, 25, 145]
    } else {
        [30, 170, 245]
    };
    if is_left {
        fill_rect(rgba, bx + pad, by + 4, lit_width, bh - 8, color);
    } else {
        fill_rect(
            rgba,
            bx + bw - 10 - lit_width,
            by + 4,
            lit_width,
            bh - 8,
            color,
        );
    }
    let _ = lw;
}

fn paint_digit(rgba: &mut [u8], x: usize, y: usize, w: usize, h: usize, digit: u8) {
    let t = 6;
    let segments = match digit {
        0 => [true, true, true, false, true, true, true],
        1 => [false, false, true, false, false, true, false],
        2 => [true, false, true, true, true, false, true],
        3 => [true, false, true, true, false, true, true],
        _ => unreachable!(),
    };
    let white = [245, 245, 245];
    if segments[0] {
        fill_rect(rgba, x, y, w, t, white);
    }
    if segments[1] {
        fill_rect(rgba, x, y, t, h / 2, white);
    }
    if segments[2] {
        fill_rect(rgba, x + w - t, y, t, h / 2, white);
    }
    if segments[3] {
        fill_rect(rgba, x, y + h / 2 - t / 2, w, t, white);
    }
    if segments[4] {
        fill_rect(rgba, x, y + h / 2, t, h / 2, white);
    }
    if segments[5] {
        fill_rect(rgba, x + w - t, y + h / 2, t, h / 2, white);
    }
    if segments[6] {
        fill_rect(rgba, x, y + h - t, w, t, white);
    }
}

fn fill_rect(rgba: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 3]) {
    for py in y..(y + height).min(HEIGHT) {
        for px in x..(x + width).min(WIDTH) {
            let index = (py * WIDTH + px) * 4;
            rgba[index..index + 3].copy_from_slice(&color);
            rgba[index + 3] = 255;
        }
    }
}
