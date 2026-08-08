//! 数字の描き込み。
//!
//! 桁の照合はテンプレートとの正規化相関で行うので、読み取りを検査するには
//! 「その数字に見える画素」が要る。テンプレートそのものを描き込むのが最も
//! 素直で、照合器が返す答えを一意に決められる。攻撃情報表示の読み取りも
//! 同じ照合器を使うため、この補助を crate の外へ公開する。

use crate::input_history::{DIGIT_H, DIGIT_NCC, DIGIT_W};

/// 数字 1 桁のテンプレート幅。
pub const GLYPH_WIDTH: usize = DIGIT_W;
/// 数字 1 桁のテンプレート高さ。
pub const GLYPH_HEIGHT: usize = DIGIT_H;

/// テンプレートどおりの数字を RGBA バッファへ描き込む。
///
/// テンプレートの左上が `(x0, y0)` に来る。照合はこの位置を起点に行うので、
/// 読み取り側も同じ位置で `match_digit_gray` を呼ぶ必要がある。
///
/// マスクの立っていない画素には触れない（背景は呼び出し側の責任）。
pub fn paint_digit(rgba: &mut [u8], width: usize, x0: usize, y0: usize, digit: usize) {
    let (mask, means) = &DIGIT_NCC[digit];
    for y in 0..DIGIT_H {
        for (x, &value) in means[y].iter().enumerate() {
            if mask[y] & (1 << x) == 0 {
                continue;
            }
            let index = ((y0 + y) * width + x0 + x) * 4;
            let Some(pixel) = rgba.get_mut(index..index + 4) else {
                continue;
            };
            pixel[0] = value;
            pixel[1] = value;
            pixel[2] = value;
            pixel[3] = 255;
        }
    }
}

/// 描き込んだ数字のうち、白判定を通る最初の画素の位置を
/// テンプレート左上からの相対位置で返す。
///
/// 連結成分で桁を切り出す読み取り側は、明るい画素の外接矩形の 1 つ外側を
/// 起点として照合する。つまりここが `(1, 1)` を返す数字だけが、描き込んだ
/// 位置とちょうど同じ起点で照合される。
pub fn bright_origin(digit: usize, white_threshold: u8) -> (usize, usize) {
    let (mask, means) = &DIGIT_NCC[digit];
    let (mut x0, mut y0) = (DIGIT_W, DIGIT_H);
    for y in 0..DIGIT_H {
        for (x, &value) in means[y].iter().enumerate() {
            if mask[y] & (1 << x) == 0 || value <= white_threshold {
                continue;
            }
            x0 = x0.min(x);
            y0 = y0.min(y);
        }
    }
    (x0, y0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_history::{match_digit_gray, Frame};

    /// 描き込んだ数字は、同じ起点で照合すれば必ずその数字として読める。
    /// ここが崩れると、この補助を使うテストは全て無意味になる。
    #[test]
    fn every_painted_digit_reads_back_as_itself() {
        let width = 64;
        for digit in 0..10 {
            let mut rgba = vec![0u8; width * 32 * 4];
            paint_digit(&mut rgba, width, 4, 4, digit);
            let frame = Frame::new(&rgba, width, 0, 210);

            let (read, score, margin) = match_digit_gray(&frame, 4, 4);

            assert_eq!(read, digit as u32, "{digit} を描いたのに {read} と読めた");
            assert_eq!(score, 0, "テンプレートそのものなので誤差は出ない");
            assert!(margin > 0, "2 位と差が付かない");
        }
    }

    /// マスクの外へはみ出して塗らない。連結成分で桁を切り出す読み取りは
    /// 明るい画素の形から桁の位置を決めるので、余分な塗りがあると外接矩形が
    /// テンプレート全体まで広がり、桁として認識されなくなる。
    #[test]
    fn painting_leaves_unmasked_pixels_untouched() {
        let width = 64;
        let mut rgba = vec![7u8; width * 32 * 4];

        paint_digit(&mut rgba, width, 4, 4, 7);

        let (mask, _) = &DIGIT_NCC[7];
        let mut painted = 0;
        for (y, &row) in mask.iter().enumerate() {
            for x in 0..GLYPH_WIDTH {
                let index = ((4 + y) * width + 4 + x) * 4;
                let touched = rgba[index] != 7;
                assert_eq!(
                    touched,
                    row & (1 << x) != 0,
                    "({x}, {y}) の塗りがマスクと食い違う"
                );
                painted += usize::from(touched);
            }
        }
        assert!(
            painted < GLYPH_WIDTH * GLYPH_HEIGHT,
            "マスクが全面なら塗り範囲を確かめられない"
        );
    }

    /// 連結成分で桁を切り出す読み取りが素直に扱えるのは、明るい画素が
    /// テンプレートの縁から 1px 内側に収まる数字だけ。どれがそうなのかを
    /// ここで固定しておく（攻撃情報表示のテストがこの事実に依存する）。
    #[test]
    fn only_some_digits_sit_exactly_one_pixel_inside_their_template() {
        let inset: Vec<usize> = (0..10)
            .filter(|&digit| bright_origin(digit, 210) == (1, 1))
            .collect();

        assert_eq!(inset, vec![7]);
    }
}
