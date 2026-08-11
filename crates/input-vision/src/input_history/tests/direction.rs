//! 入力履歴の方向グリフを読むところに対するテスト。
//!
//! 方向は 9 種類の小さな白い矢印で表示される。背景は透けるし、
//! ヒット数の表示やスーパーの演出が裏に重なる。だから「一番近い
//! テンプレート」だけでは足りない。二番目との差が開いていることまで
//! 確かめて、初めてその方向だと言える。

use super::super::direction::{
    alignment_offset, dir_mask, direction_score_is_accepted, fine_offsets, mask_centroid,
    rank_direction_candidate, read_dir, shift_mask, within_alignment_window,
};
use super::super::*;

const WHITE_THRESHOLD: u8 = 210;

/// 1 枚のグリフだけが描かれた画面。
struct Canvas {
    rgba: Vec<u8>,
    width: usize,
    height: usize,
}

impl Canvas {
    fn new() -> Self {
        let (width, height) = (DIR_W + 20, DIR_H + 20);
        Self {
            rgba: vec![0u8; width * height * 4],
            width,
            height,
        }
    }

    fn set(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = (y * self.width + x) * 4;
        self.rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
    }

    /// マスクをそのまま (0, 0) 起点で描く。
    fn draw(&mut self, mask: &[u64; DIR_H]) {
        for (y, bits) in mask.iter().enumerate() {
            for x in 0..DIR_W {
                if bits & (1 << x) != 0 {
                    self.set(x, y);
                }
            }
        }
    }

    fn read(&self) -> (InputDir, bool, u32) {
        let frame = Frame::new(&self.rgba, self.width, 0, WHITE_THRESHOLD);
        read_dir(&frame, 0, 0)
    }
}

fn template_for(direction: InputDir) -> [u64; DIR_H] {
    let index = DIR_ORDER
        .iter()
        .position(|candidate| *candidate == direction)
        .expect("方向にテンプレートがある");
    DIR_TEMPLATES[index]
}

fn painted(mask: &[u64; DIR_H]) -> Canvas {
    let mut canvas = Canvas::new();
    canvas.draw(mask);
    canvas
}

// ── 白い点の集まりとして見る ─────────────────────────────────────────────

/// 白い画素の位置と数を拾う。
#[test]
fn the_mask_collects_the_white_pixels() {
    let mut canvas = Canvas::new();
    canvas.set(3, 2);
    canvas.set(5, 2);
    canvas.set(3, 7);
    canvas.set(DIR_W, 7);
    let frame = Frame::new(&canvas.rgba, canvas.width, 0, WHITE_THRESHOLD);

    let (mask, count) = dir_mask(&frame, 0, 0);

    assert_eq!(count, 3);
    assert_eq!(mask[2], (1 << 3) | (1 << 5));
    assert_eq!(mask[7], 1 << 3);
    assert_eq!(mask[0], 0);
}

#[test]
fn alignment_helpers_have_exact_rounding_search_and_score_boundaries() {
    assert_eq!(alignment_offset((4.6, 3.4), (2.2, 1.7)), (2, 2));
    assert_eq!(alignment_offset((1.2, 2.1), (3.6, 5.8)), (-2, -4));
    assert_eq!(fine_offsets(4), [3, 4, 5]);
    assert_eq!(fine_offsets(-4), [-5, -4, -3]);

    assert!(within_alignment_window(6, -6));
    assert!(!within_alignment_window(7, 0));
    assert!(!within_alignment_window(0, -7));

    let best = (InputDir::Left, 5);
    assert_eq!(
        rank_direction_candidate(best, 9, (InputDir::Right, 4)),
        ((InputDir::Right, 4), 5)
    );
    assert_eq!(
        rank_direction_candidate(best, 9, (InputDir::Right, 5)),
        (best, 5)
    );
    assert_eq!(
        rank_direction_candidate(best, 9, (InputDir::Right, 7)),
        (best, 7)
    );
    assert_eq!(
        rank_direction_candidate(best, 9, (InputDir::Right, 9)),
        (best, 9)
    );

    assert!(direction_score_is_accepted(32, 40));
    assert!(!direction_score_is_accepted(33, 41));
    assert!(!direction_score_is_accepted(32, 39));
}

/// 重心は白い点の平均の位置。
#[test]
fn the_centroid_is_the_average_position_of_the_white_pixels() {
    let mut mask = [0u64; DIR_H];
    mask[2] = (1 << 4) | (1 << 8);
    mask[6] = 1 << 8;

    let (x, y) = mask_centroid(&mask).expect("白があれば重心がある");

    assert!((x - 20.0 / 3.0).abs() < 1e-5, "x={x}");
    assert!((y - 10.0 / 3.0).abs() < 1e-5, "y={y}");
}

/// 白が一つも無ければ重心は無い。
#[test]
fn an_empty_mask_has_no_centroid() {
    assert!(mask_centroid(&[0u64; DIR_H]).is_none());
}

// ── ずらす ───────────────────────────────────────────────────────────────

/// 右下へずらす。
#[test]
fn shifting_moves_the_mask_right_and_down() {
    let mut mask = [0u64; DIR_H];
    mask[3] = 1 << 5;

    let shifted = shift_mask(&mask, 2, 4);

    assert_eq!(shifted[7], 1 << 7);
    assert_eq!(shifted[3], 0);
}

/// 左上へもずらす。
#[test]
fn shifting_moves_the_mask_left_and_up() {
    let mut mask = [0u64; DIR_H];
    mask[10] = 1 << 9;

    let shifted = shift_mask(&mask, -3, -6);

    assert_eq!(shifted[4], 1 << 6);
    assert_eq!(shifted[10], 0);
}

/// はみ出した分は捨てる。回り込ませない。
#[test]
fn what_is_shifted_off_the_edge_is_dropped() {
    let mut mask = [0u64; DIR_H];
    mask[0] = 1 << 1;
    mask[DIR_H - 1] = 1 << (DIR_W - 2);

    let up = shift_mask(&mask, 0, -2);
    let right = shift_mask(&mask, 3, 0);

    assert_eq!(up[DIR_H - 3], 1 << (DIR_W - 2));
    assert!(
        up.iter().all(|row| *row & (1 << 1) == 0),
        "上へ回り込んでいる"
    );
    assert_eq!(
        right[DIR_H - 1],
        0,
        "右端からはみ出した点が残っている: {:b}",
        right[DIR_H - 1]
    );
    assert_eq!(right[0], 1 << 4);
}

// ── 方向を決める ─────────────────────────────────────────────────────────

/// テンプレートそのものを描けば、その方向として読める。
#[test]
fn every_direction_reads_back_from_its_own_glyph() {
    for direction in DIR_ORDER {
        let (read, uncertain, _) = painted(&template_for(direction)).read();

        assert_eq!(read, direction, "{direction:?} を読み違えている");
        assert!(!uncertain, "{direction:?} を不確定にしている");
    }
}

/// 数ピクセルずれていても読める。表示位置は行ごとに揺れる。
#[test]
fn a_glyph_shifted_by_a_few_pixels_still_reads() {
    for (dx, dy) in [(2, 0), (0, 2), (-2, 2), (3, -3)] {
        let shifted = shift_mask(&template_for(InputDir::DownRight), dx, dy);
        let mut canvas = Canvas::new();
        for (y, bits) in shifted.iter().enumerate() {
            for x in 0..DIR_W {
                if bits & (1 << x) != 0 {
                    canvas.set(x, y);
                }
            }
        }

        assert_eq!(
            canvas.read().0,
            InputDir::DownRight,
            "({dx}, {dy}) のずれで読めていない"
        );
    }
}

/// 白がほとんど無い行は、入力が無かった行。読めなかったのではない。
#[test]
fn a_row_with_almost_no_white_is_empty_not_unreadable() {
    let mut canvas = Canvas::new();
    for x in 0..10 {
        canvas.set(x, 5);
    }

    let (direction, uncertain, _) = canvas.read();

    assert_eq!(direction, InputDir::Unknown);
    assert!(!uncertain, "空の行を「読めなかった」にしている");
}

/// 一面が白く飛んでいる行は、読めなかった行。入力が無かったのではない。
#[test]
fn a_row_washed_out_in_white_is_unreadable_not_empty() {
    let mut canvas = Canvas::new();
    for y in 0..DIR_H {
        for x in 0..DIR_W {
            canvas.set(x, y);
        }
    }

    let (direction, uncertain, _) = canvas.read();

    assert_eq!(direction, InputDir::Unknown);
    assert!(uncertain, "白飛びを空の行にしている");
}

/// どのテンプレートにも似ていない形は読めない。
#[test]
fn a_shape_unlike_every_glyph_is_not_guessed() {
    // 左端に寄せた細い縦棒。矢印のどれとも重ならない。
    let mut canvas = Canvas::new();
    for y in 0..DIR_H {
        for x in 0..3 {
            canvas.set(x, y);
        }
    }

    let (direction, uncertain, _) = canvas.read();

    assert_eq!(direction, InputDir::Unknown);
    assert!(uncertain);
}

/// 二つのグリフを重ねると、どちらとも決まらない。近いだけでは足りず、
/// 二番目との差が要る。
#[test]
fn a_shape_between_two_glyphs_is_left_unknown() {
    let mut merged = template_for(InputDir::Up);
    for (row, bits) in template_for(InputDir::UpRight).iter().enumerate() {
        merged[row] |= bits;
    }

    let (direction, uncertain, _) = painted(&merged).read();

    assert_eq!(direction, InputDir::Unknown, "重なった形を断定している");
    assert!(uncertain);
}

/// 白の数がいくつあれば「行がある」と見なすか。少なすぎれば入力の
/// 無い行、多すぎれば白飛び。どちらもグリフではない。
#[test]
fn the_amount_of_white_decides_empty_from_washed_out() {
    let read_with_white = |count: usize| {
        let mut canvas = Canvas::new();
        for index in 0..count {
            canvas.set(index % DIR_W, index / DIR_W);
        }
        canvas.read()
    };

    let (_, few_uncertain, _) = read_with_white(39);
    assert!(!few_uncertain, "白が少ない行を「読めなかった」にしている");

    let (_, enough_uncertain, _) = read_with_white(40);
    assert!(enough_uncertain, "白が足りている行を空にしている");

    let (_, wide_uncertain, _) = read_with_white(701);
    assert!(wide_uncertain, "白飛びを空の行にしている");

    let (_, exact_uncertain, exact_score) = read_with_white(700);
    assert!(exact_uncertain);
    assert_ne!(exact_score, u32::MAX, "上限ちょうどを早期棄却している");
}
