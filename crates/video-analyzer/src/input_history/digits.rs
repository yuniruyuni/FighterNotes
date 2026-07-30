use super::*;

// ── 数字読み取り ─────────────────────────────────────────────────────────────

/// 1 桁を per-pixel 統計モデルで照合。(digit, score, 2位とのマージン) を返す。
///
/// 各桁の安定画素（サンプル間で値がほぼ一定 = グリフ本体と輪郭）上で
/// サンプルとテンプレート平均の正規化相関を取る。輝度・コントラスト
/// 不変なので画面暗転にも単一パスで対応できる。
/// score = (1 - 相関) × 100（小さいほど良い）。±1px シフトも試す。
pub(crate) fn match_digit_gray(f: &Frame, x0: usize, y0: usize) -> (u32, u32, u32) {
    let mut best = (0u32, u32::MAX);
    let mut second = (0u32, u32::MAX);
    for (d, (mask, means)) in DIGIT_NCC.iter().enumerate() {
        let (mut sum_t, mut sum_tt, mut n) = (0i64, 0i64, 0i64);
        for y in 0..DIGIT_H {
            let mrow = mask[y];
            for (x, &template_value) in means[y].iter().enumerate() {
                if mrow & (1 << x) == 0 {
                    continue;
                }
                let value = i64::from(template_value);
                sum_t += value;
                sum_tt += value * value;
                n += 1;
            }
        }
        let mut sbest = u32::MAX;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (mut sum_s, mut sum_ss, mut sum_st) = (0i64, 0i64, 0i64);
                for y in 0..DIGIT_H {
                    let mrow = mask[y];
                    if mrow == 0 {
                        continue;
                    }
                    for (x, &template_value) in means[y].iter().enumerate() {
                        if mrow & (1 << x) == 0 {
                            continue;
                        }
                        let sx = (x0 as i32 + x as i32 + dx).max(0) as usize;
                        let sy = (y0 as i32 + y as i32 + dy).max(0) as usize;
                        let sv = i64::from(f.gray(sx, sy));
                        let tv = i64::from(template_value);
                        sum_s += sv;
                        sum_ss += sv * sv;
                        sum_st += sv * tv;
                    }
                }
                if n < 20 {
                    continue;
                }
                // Pearson相関を和だけで計算する。平均との差分を配列へ保存して
                // 2周目を回す式と等価で、一時配列と再走査を避けられる。
                let numerator = n * sum_st - sum_s * sum_t;
                let sample_variance = n * sum_ss - sum_s * sum_s;
                let template_variance = n * sum_tt - sum_t * sum_t;
                if sample_variance <= 0 || template_variance <= 0 {
                    continue;
                }
                let denominator = ((sample_variance as f64) * (template_variance as f64)).sqrt();
                let r = numerator as f64 / denominator;
                let score = ((1.0 - r) * 100.0).max(0.0) as u32;
                sbest = sbest.min(score);
            }
        }
        if sbest < best.1 {
            second = best;
            best = (d as u32, sbest);
        } else if sbest < second.1 {
            second = (d as u32, sbest);
        }
    }
    (best.0, best.1, second.1.saturating_sub(best.1))
}

/// 桁マッチの許容スコア（(1-相関)×100）。正解 実測 ≤25（背景次第で
/// 細身の '1' 等が 23-25 まで上がる）、空ボックスの背景ノイズは ≥58。
/// 誤読は曖昧マージン（<3）が防ぐため、受理はゆるめでよい
const DIGIT_MAX_DIFF: u32 = 28;
/// マージン受理の上限スコア。'1' はマスクに背景画素が多く、明るい背景で
/// 正解スコアが 30-34 まで膨らむ（実測）。2 位と大差の 1 位はここまで許容
const DIGIT_MARGIN_MAX_DIFF: u32 = 40;
/// マージン受理に必要な 2 位との差。存在ゲート通過空箱の実測は
/// score ≤ 40 帯でマージン ≤ 4 に留まり、この条件を満たす偽物は
/// 観測されていない。正解側の実測マージンは最小 18
const DIGIT_MARGIN_MIN: u32 = 15;
/// 1 位と 2 位のマージンがこれ未満なら ambiguous（描画劣化で原理的に
/// 曖昧）として上位層（count 連続性トラッカー）に委ねる
pub(super) const DIGIT_AMBIG_MARGIN: u32 = 3;
/// 桁ボックスに数字があると見なす最小白ピクセル数
const DIGIT_MIN_WHITE: u32 = 12;

/// カウント数字列を読む。右端（ones）から左へ最大 MAX_DIGITS 桁。
/// 戻り値: (count, uncertain, スコア合計)。スコアが小さいほど確信度が高い。
///
/// 桁の存在判定は低閾値（180）の白画素数で行い（暗転時も文字は 180 超）、
/// 分類は正規化相関（輝度不変）で行う。
pub(super) fn read_count(f: &Frame, ones_x: u32, y0: usize) -> (Option<u32>, bool, u32) {
    let mut value = 0u32;
    let mut scale = 1u32;
    let mut digits = 0usize;
    let mut total_score = 0u32;
    for k in 0..MAX_DIGITS {
        let x0 = ones_x as i64 - (k as i64) * DIGIT_W as i64;
        if x0 < 0 {
            break;
        }
        let x0 = x0 as usize;
        // 桁存在判定: グリフの白芯（240+）は背景（透過部のキャラ肌 ≤220 等）より
        // 明るいため、強証拠は >230。暗転時はグリフ全体が沈むため、弱証拠
        // （>180）+ NCC 一致でも桁ありとする。
        // パネルの上に他の演出が重なることはない（グリフは常に不透明）ので、
        // 弱証拠のみで NCC 全滅なら「桁なし」（背景ノイズ）と断定できる
        let (mut n_strong, mut n_weak) = (0u32, 0u32);
        for y in 0..DIGIT_H {
            for x in 0..DIGIT_W {
                let v = f.gray(x0 + x, y0 + y);
                if v > 230 {
                    n_strong += 1;
                }
                if v > 180 {
                    n_weak += 1;
                }
            }
        }
        let strong = n_strong >= 8;
        if !strong && n_weak < DIGIT_MIN_WHITE {
            break; // 桁なし = 数字列の終端
        }
        let (d, score, margin) = match_digit_gray(f, x0, y0);
        let accepted = score <= DIGIT_MAX_DIFF
            || (score <= DIGIT_MARGIN_MAX_DIFF && margin >= DIGIT_MARGIN_MIN);
        if !accepted {
            if strong {
                return (None, true, u32::MAX); // 白芯があるのに読めない = 想定外
            }
            break; // 弱証拠のみ + NCC 全滅 = 背景ノイズ → 桁なし
        }
        if margin < DIGIT_AMBIG_MARGIN {
            return (None, true, u32::MAX); // 描画劣化で原理的に曖昧
        }
        value += d * scale;
        scale *= 10;
        digits += 1;
        total_score += score;
    }
    if digits == 0 {
        (None, false, u32::MAX) // 空行（数字なし）
    } else {
        (Some(value), false, total_score)
    }
}
