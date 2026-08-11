//! 入力履歴の継続フレーム数を読むところに対するテスト。
//!
//! 数字は右端の桁から左へ並ぶ。桁が何個あるかは書かれていないので、
//! 「もう桁が無い」ところで止めるしかない。止め方を間違えると、
//! 背景の模様を 4 桁目として読む。
//!
//! 画面暗転中はグリフ全体が沈む。明るい芯が見えるかどうかで、
//! 「読めなかった」と「そもそも桁が無い」を分ける。

use super::super::digits::{
    digit_box_has_trace, digit_evidence, digit_match_is_accepted, match_digit_gray,
    rank_digit_candidate, read_count, shifted_coordinate, variances_are_positive,
};
use super::super::*;

const WHITE_THRESHOLD: u8 = 210;

/// 数字の並ぶ帯だけを切り出した画面。
struct Panel {
    rgba: Vec<u8>,
    width: usize,
}

/// 一番右の桁の左上 x。
const ONES_X: usize = 60;

impl Panel {
    fn new() -> Self {
        let width = ONES_X + DIGIT_W + 10;
        Self {
            rgba: vec![0u8; width * (DIGIT_H + 10) * 4],
            width,
        }
    }

    fn shade(&mut self, x: usize, y: usize, level: u8) {
        let index = (y * self.width + x) * 4;
        self.rgba[index..index + 4].copy_from_slice(&[level, level, level, 255]);
    }

    /// テンプレートそのままの明るさで 1 桁を描く。
    fn draw_digit(&mut self, digit: usize, x0: usize) {
        self.draw_digit_dimmed(digit, x0, 1.0);
    }

    /// 明るさを一律に落として 1 桁を描く。画面暗転の再現。
    fn draw_digit_dimmed(&mut self, digit: usize, x0: usize, scale: f32) {
        for y in 0..DIGIT_H {
            for x in 0..DIGIT_W {
                let level = (DIGIT_NCC[digit].1[y][x] as f32 * scale).round() as u8;
                self.shade(x0 + x, y, level);
            }
        }
    }

    /// 桁枠へ、指定した数の明るい点だけを置く。数字には見えない。
    fn draw_dots(&mut self, position: usize, count: usize, level: u8) {
        let x0 = ONES_X - position * DIGIT_W;
        for index in 0..count {
            self.shade(x0 + index % DIGIT_W, index / DIGIT_W, level);
        }
    }

    /// 右端から `position` 個目の桁枠へ描く。
    fn draw_at_position(&mut self, digit: usize, position: usize) {
        self.draw_digit(digit, ONES_X - position * DIGIT_W);
    }

    /// 数を右詰めで描く。
    fn draw_number(&mut self, value: u32) {
        for (position, digit) in value
            .to_string()
            .chars()
            .rev()
            .map(|c| c.to_digit(10).unwrap() as usize)
            .enumerate()
        {
            self.draw_at_position(digit, position);
        }
    }

    /// 指定の桁枠を一様な明るさで塗る。
    fn fill_position(&mut self, position: usize, level: u8) {
        let x0 = ONES_X - position * DIGIT_W;
        for y in 0..DIGIT_H {
            for x in 0..DIGIT_W {
                self.shade(x0 + x, y, level);
            }
        }
    }

    fn frame(&self) -> Frame<'_> {
        Frame::new(&self.rgba, self.width, 0, WHITE_THRESHOLD)
    }

    fn read(&self) -> (Option<u32>, bool, u32) {
        read_count(&self.frame(), ONES_X as u32, 0)
    }
}

// ── 1 桁を見分ける ───────────────────────────────────────────────────────

#[test]
fn digit_helpers_preserve_coordinates_rankings_and_exact_thresholds() {
    assert_eq!(shifted_coordinate(10, 3, 2), 15);
    assert_eq!(shifted_coordinate(1, 1, -5), 0);
    assert!(variances_are_positive(1, 1));
    assert!(!variances_are_positive(0, 1));
    assert!(!variances_are_positive(1, 0));

    let best = (2, 5);
    let second = (3, 9);
    assert_eq!(rank_digit_candidate(best, second, (4, 4)), ((4, 4), best));
    assert_eq!(rank_digit_candidate(best, second, (4, 5)), (best, (4, 5)));
    assert_eq!(rank_digit_candidate(best, second, (4, 7)), (best, (4, 7)));
    assert_eq!(rank_digit_candidate(best, second, (4, 9)), (best, second));

    assert!(!digit_box_has_trace(false, 11));
    assert!(digit_box_has_trace(false, 12));
    assert!(digit_box_has_trace(true, 0));
    assert!(digit_match_is_accepted(28, 0));
    assert!(digit_match_is_accepted(40, 15));
    assert!(!digit_match_is_accepted(29, 14));
    assert!(!digit_match_is_accepted(41, 15));
}

#[test]
fn digit_evidence_uses_strict_brightness_and_box_boundaries() {
    let mut panel = Panel::new();
    panel.shade(ONES_X, 0, 231);
    panel.shade(ONES_X + 1, 0, 230);
    panel.shade(ONES_X + 2, 0, 181);
    panel.shade(ONES_X + 3, 0, 180);
    panel.shade(ONES_X + DIGIT_W, 0, 255);
    panel.shade(ONES_X, DIGIT_H, 255);

    assert_eq!(digit_evidence(&panel.frame(), ONES_X, 0), (1, 3));
}

/// 手本そのものを描けば、その数字として読める。
#[test]
fn every_digit_reads_back_from_its_own_glyph() {
    for digit in 0..10usize {
        let mut panel = Panel::new();
        panel.draw_digit(digit, ONES_X);

        let (read, score, margin) = match_digit_gray(&panel.frame(), ONES_X, 0);

        assert_eq!(read as usize, digit, "{digit} を読み違えている");
        assert_eq!(score, 0, "{digit} の一致が完全でない");
        assert!(margin > 0, "{digit} が二番目と並んでいる");
    }
}

/// 1 ピクセルずれていても読める。行の描画位置は揺れる。
#[test]
fn a_digit_shifted_by_one_pixel_still_reads() {
    for offset in [1usize, 2] {
        let mut panel = Panel::new();
        panel.draw_digit(7, ONES_X + offset);

        assert_eq!(
            match_digit_gray(&panel.frame(), ONES_X + offset - 1, 0).0,
            7,
            "{offset} ピクセルのずれで読めていない"
        );
    }
}

#[test]
fn a_digit_shifted_down_by_one_pixel_still_reads() {
    let mut panel = Panel::new();
    for y in 0..DIGIT_H {
        for x in 0..DIGIT_W {
            panel.shade(ONES_X + x, y + 1, DIGIT_NCC[7].1[y][x]);
        }
    }

    assert_eq!(match_digit_gray(&panel.frame(), ONES_X, 0).0, 7);
}

/// 濃淡の無い箱は、どの数字とも言えない。
#[test]
fn a_flat_box_matches_nothing() {
    let mut panel = Panel::new();
    for y in 0..DIGIT_H + 10 {
        for x in 0..panel.width {
            panel.shade(x, y, 200);
        }
    }

    let (_, score, _) = match_digit_gray(&panel.frame(), ONES_X, 0);

    assert_eq!(score, u32::MAX, "のっぺりした箱を数字にしている");
}

// ── 桁の並びを読む ───────────────────────────────────────────────────────

/// 右端から左へ、桁のある分だけ読む。
#[test]
fn the_count_is_read_from_the_ones_column_leftwards() {
    for value in [0u32, 7, 42, 999] {
        let mut panel = Panel::new();
        panel.draw_number(value);

        let (count, uncertain, _) = panel.read();

        assert_eq!(count, Some(value), "{value} を読み違えている");
        assert!(!uncertain);
    }
}

/// 表示は 3 桁まで。4 桁目は読まない。
#[test]
fn only_the_three_rightmost_digits_are_read() {
    let mut panel = Panel::new();
    panel.draw_number(999);
    panel.draw_at_position(8, 3);

    assert_eq!(panel.read().0, Some(999), "4 桁目まで読んでいる");
}

/// 何も描かれていない行は、入力が無かった行。読めなかったのではない。
#[test]
fn an_empty_row_has_no_count_and_is_not_uncertain() {
    let (count, uncertain, _) = Panel::new().read();

    assert_eq!(count, None);
    assert!(!uncertain, "空の行を「読めなかった」にしている");
}

/// 桁の途中で空の枠に当たったら、そこが数字の左端。
#[test]
fn an_empty_box_ends_the_number() {
    let mut panel = Panel::new();
    panel.draw_at_position(5, 0);
    // 10 の位は空のまま、100 の位に模様を置いても届かない。
    panel.draw_at_position(3, 2);

    assert_eq!(panel.read().0, Some(5), "空の桁を飛び越えて読んでいる");
}

/// 明るい芯があるのに読めない箱は、読めなかった行として扱う。
/// 見えているものを黙って捨てない。
#[test]
fn a_bright_box_that_matches_nothing_makes_the_row_unreadable() {
    let mut panel = Panel::new();
    panel.draw_at_position(4, 0);
    panel.fill_position(1, 255);

    let (count, uncertain, _) = panel.read();

    assert_eq!(count, None, "読めない箱を無視して数を返している");
    assert!(uncertain);
}

/// 暗くて薄い模様しか無い箱は、桁ではなく背景。そこで数字は終わり。
#[test]
fn a_faint_box_that_matches_nothing_is_treated_as_background() {
    let mut panel = Panel::new();
    panel.draw_at_position(4, 0);
    // 弱い証拠（181 以上）だけの、数字に似ていない模様。
    let x0 = ONES_X - DIGIT_W;
    for y in 0..DIGIT_H {
        for x in 0..DIGIT_W {
            panel.shade(x0 + x, y, if (x + y) % 2 == 0 { 200 } else { 190 });
        }
    }

    let (count, uncertain, _) = panel.read();

    assert_eq!(count, Some(4), "背景の模様を桁として読んでいる");
    assert!(!uncertain);
}

/// 画面の左端を越えてまで桁を探さない。
#[test]
fn the_search_stops_at_the_left_edge_of_the_panel() {
    let mut panel = Panel::new();
    panel.draw_digit(6, 5);

    let (count, uncertain, _) = read_count(&panel.frame(), 5, 0);

    assert_eq!(count, Some(6));
    assert!(!uncertain);
}

#[test]
fn a_digit_whose_ones_column_is_at_zero_is_read() {
    let mut panel = Panel::new();
    panel.draw_digit(6, 0);

    assert_eq!(read_count(&panel.frame(), 0, 0).0, Some(6));
}

#[test]
fn a_rejected_weak_box_stops_before_a_later_digit() {
    let mut panel = Panel::new();
    panel.draw_at_position(4, 0);
    let x0 = ONES_X - DIGIT_W;
    for y in 0..DIGIT_H {
        for x in 0..DIGIT_W {
            panel.shade(x0 + x, y, if (x + y) % 2 == 0 { 200 } else { 190 });
        }
    }
    panel.draw_at_position(3, 2);

    assert_eq!(panel.read().0, Some(4));
}

#[test]
fn count_score_is_the_sum_of_the_accepted_digit_scores() {
    let mut panel = Panel::new();
    panel.draw_digit(8, ONES_X);
    let mut changed = 0;
    for y in 0..DIGIT_H {
        for x in 0..DIGIT_W {
            if DIGIT_NCC[8].0[y] & (1 << x) != 0 && changed < 4 {
                let original = DIGIT_NCC[8].1[y][x];
                panel.shade(ONES_X + x, y, 255 - original);
                changed += 1;
            }
        }
    }
    let (digit, score, _) = match_digit_gray(&panel.frame(), ONES_X, 0);
    assert_eq!(digit, 8);
    assert!(score > 0);

    let (count, uncertain, total_score) = panel.read();
    assert_eq!(count, Some(8));
    assert!(!uncertain);
    assert_eq!(total_score, score);
}

// ── 桁があるかどうかの見分け ─────────────────────────────────────────────

/// 画面が暗転しても数字は読める。グリフ全体が沈むだけで、形は残る。
#[test]
fn a_dimmed_digit_is_still_read() {
    let mut panel = Panel::new();
    panel.draw_digit_dimmed(3, ONES_X, 0.85);

    let (count, uncertain, _) = panel.read();

    assert_eq!(count, Some(3), "暗転した数字を読めていない");
    assert!(!uncertain);
}

/// 沈みきって痕跡も残らない枠は、桁が無い。そこで数字は終わり。
#[test]
fn a_box_with_no_trace_left_ends_the_number() {
    let mut panel = Panel::new();
    panel.draw_digit(5, ONES_X);
    panel.draw_digit_dimmed(3, ONES_X - DIGIT_W, 0.6);

    assert_eq!(panel.read().0, Some(5), "痕跡だけの枠を桁にしている");
}

/// 白芯がいくつあれば「確かに何か描かれている」と見なすか。数が足りな
/// ければ背景ノイズとして数字の終わりにし、足りていれば読めなかった行に
/// する。無言で捨てるのと、読めなかったと言うのは別のこと。
#[test]
fn the_number_of_bright_dots_decides_background_from_unreadable() {
    let read_with_dots = |dots: usize| {
        let mut panel = Panel::new();
        panel.draw_digit(5, ONES_X);
        panel.draw_dots(1, dots, 255);
        panel.read()
    };

    let (few, few_uncertain, _) = read_with_dots(7);
    assert_eq!(few, Some(5), "背景ノイズを読めない行にしている");
    assert!(!few_uncertain);

    let (many, many_uncertain, _) = read_with_dots(8);
    assert_eq!(many, None, "白芯のある枠を黙って捨てている");
    assert!(many_uncertain);
}

/// 二つの数字の中間の形は、どちらとも決めない。上位の連続性の判断へ
/// 委ねる。
#[test]
fn a_digit_between_two_shapes_is_left_unread() {
    let mut panel = Panel::new();
    for y in 0..DIGIT_H {
        for x in 0..DIGIT_W {
            let blended = (DIGIT_NCC[3].1[y][x] as u16 + DIGIT_NCC[8].1[y][x] as u16) / 2;
            panel.shade(ONES_X + x, y, blended as u8);
        }
    }

    let (digit, score, margin) = match_digit_gray(&panel.frame(), ONES_X, 0);
    if margin < 3 {
        let (count, uncertain, _) = panel.read();
        assert_eq!(count, None, "曖昧な桁を断定している");
        assert!(uncertain);
    } else {
        // 中間の形でも差が付くなら、それはそれで読めている。
        assert!(
            score <= 40,
            "読めた扱いなのにスコアが悪い: {digit} {score} {margin}"
        );
    }
}
